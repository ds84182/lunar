use std::{borrow::Borrow, marker::PhantomData, mem::offset_of, ops::Bound, ptr::NonNull};

use crate::utils::{AllocError, LuaDrop};

use super::GlobalState;

#[repr(C)]
struct Node<K, V> {
    link: NodeLink<K>,
    value: V,
}

#[repr(C)]
struct NodeLink<K> {
    link: wavltree::Links<Self>,
    key: K,
}

unsafe impl<K: Ord> wavltree::Linked for NodeLink<K> {
    type Handle = NonNull<NodeLink<K>>;
    type Key = K;

    #[inline]
    fn into_ptr(r: Self::Handle) -> NonNull<Self> {
        r
    }

    #[inline]
    unsafe fn from_ptr(ptr: NonNull<Self>) -> Self::Handle {
        ptr
    }

    #[inline]
    unsafe fn links(ptr: NonNull<Self>) -> NonNull<wavltree::Links<Self>> {
        // link is the first member
        ptr.cast()
    }

    #[inline]
    fn get_key(&self) -> &Self::Key {
        &self.key
    }
}

#[repr(C)]
pub(crate) struct RBTree<K: Ord, V> {
    root: wavltree::WAVLTree<NodeLink<K>>,
    _pd: PhantomData<V>,
}

impl<K: LuaDrop + Ord, V: LuaDrop> LuaDrop for RBTree<K, V> {
    fn drop_with_state(&mut self, g: GlobalState) {
        self.clear(g);
    }
}

impl<K: Ord, V> RBTree<K, V> {
    pub(crate) fn new() -> RBTree<K, V> {
        Self {
            root: Default::default(),
            _pd: PhantomData,
        }
    }

    pub(crate) fn get<'a, Q>(&'a self, key: &Q) -> Option<&'a V>
    where
        Q: Ord,
        K: Borrow<Q>,
    {
        let cursor = self.root.find(key);
        let ptr = unsafe { cursor.get_ptr()? };
        unsafe {
            Some(
                ptr.cast::<Node<K, V>>()
                    .byte_add(offset_of!(Node<K, V>, value))
                    .cast::<V>()
                    .as_ref(),
            )
        }
    }

    pub(crate) fn get_mut<'a, Q>(&'a mut self, key: &Q) -> Option<&'a mut V>
    where
        Q: Ord,
        K: Borrow<Q>,
    {
        let cursor = self.root.find_mut(key);
        let ptr = unsafe { cursor.get_ptr()? };
        unsafe {
            Some(
                ptr.cast::<Node<K, V>>()
                    .byte_add(offset_of!(Node<K, V>, value))
                    .cast::<V>()
                    .as_mut(),
            )
        }
    }

    pub(crate) fn insert(
        &mut self,
        g: GlobalState,
        key: K,
        value: V,
    ) -> Result<Option<V>, AllocError>
    where
        K: Ord,
    {
        match self.root.entry(&key) {
            wavltree::Entry::Occupied(entry) => {
                // TODO: This is unsound. Add a get_ptr method to OccupiedEntry and upstream it.
                let ptr = NonNull::from(entry.get());
                let ptr = ptr.cast::<Node<K, V>>();
                let ptr = unsafe { ptr.byte_add(offset_of!(Node<K, V>, value)).cast::<V>() };
                return Ok(Some(unsafe { ptr.replace(value) }));
            }
            wavltree::Entry::Vacant(entry) => {
                let node = g.alloc::<Node<K, V>>().ok_or(AllocError)?;
                unsafe {
                    node.write(Node {
                        link: NodeLink {
                            link: Default::default(),
                            key,
                        },
                        value,
                    })
                };

                entry.insert_entry(node.cast::<NodeLink<K>>());
            }
        }

        Ok(None)
    }

    pub(crate) fn clear(&mut self, g: GlobalState)
    where
        K: LuaDrop,
        V: LuaDrop,
    {
        let Some(mut node) = (unsafe { self.root.take().root().get_ptr() }) else {
            return;
        };

        use wavltree::Linked;

        loop {
            // Traverse to the leftmost leaf descendant
            let mut links;
            loop {
                links = unsafe { NodeLink::links(node).as_ref() };
                let Some(child) = links.left().or(links.right()) else {
                    break;
                };
                node = child;
            }

            // Set parent's child ptr to null, dealloc chlid
            let parent = links.parent();

            let ptr = node.as_ptr() as *mut Node<K, V>;
            unsafe {
                (&mut *ptr).link.key.drop_with_state(g);
                (&mut *ptr).value.drop_with_state(g);
            }
            unsafe { g.dealloc(NonNull::new_unchecked(ptr)) };

            if let Some(parent) = parent {
                let links = unsafe { NodeLink::links(parent).as_mut() };
                if links.left() == Some(node) {
                    links.replace_left(None);
                } else {
                    links.replace_right(None);
                }
                node = parent;
            } else {
                // Root.
                break;
            }
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        // TODO: Just use Cursor api instead of iter
        self.root.iter().map(|node| {
            // TODO: This is unsound. Add iter_raw and upstream it.
            let ptr = NonNull::from(node);
            let ptr = ptr.cast::<Node<K, V>>();
            let key = unsafe { ptr.byte_add(offset_of!(Node<K, V>, link.key)).cast::<K>() };
            let value = unsafe { ptr.byte_add(offset_of!(Node<K, V>, value)).cast::<V>() };
            unsafe { (key.as_ref(), value.as_ref()) }
        })
    }

    pub(crate) fn lower_bound<'a, Q>(&'a self, bound: Bound<&Q>) -> Cursor<'a, K, V>
    where
        Q: Ord,
        K: Borrow<Q>,
    {
        Cursor {
            cursor: self.root.lower_bound(bound),
            _pd: PhantomData,
        }
    }

    pub(crate) fn upper_bound<'a, Q>(&'a self, bound: Bound<&Q>) -> Cursor<'a, K, V>
    where
        Q: Ord,
        K: Borrow<Q>,
    {
        Cursor {
            cursor: self.root.upper_bound(bound),
            _pd: PhantomData,
        }
    }
}

pub(crate) struct Cursor<'a, K: Ord, V> {
    cursor: wavltree::Cursor<'a, NodeLink<K>>,
    _pd: PhantomData<&'a V>,
}

impl<'a, K: Ord, V> Cursor<'a, K, V> {
    pub(crate) fn key(&self) -> Option<&'a K> {
        let ptr = unsafe { self.cursor.get_ptr()? };
        let ptr = ptr.cast::<Node<K, V>>();
        let key = unsafe { ptr.byte_add(offset_of!(Node<K, V>, link.key)).cast::<K>() };
        unsafe { Some(key.as_ref()) }
    }

    pub(crate) fn value(&self) -> Option<&'a V> {
        let ptr = unsafe { self.cursor.get_ptr()? };
        let ptr = ptr.cast::<Node<K, V>>();
        let value = unsafe { ptr.byte_add(offset_of!(Node<K, V>, value)).cast::<V>() };
        unsafe { Some(value.as_ref()) }
    }

    pub(crate) fn is_null(&self) -> bool {
        unsafe { self.cursor.get_ptr().is_none() }
    }

    pub(crate) fn move_next(&mut self) {
        self.cursor.move_next();
    }

    pub(crate) fn move_prev(&mut self) {
        self.cursor.move_prev();
    }
}
