

///
/// Cell of an Arena.
///
#[derive(Debug)]
pub struct ArenaCell<T>{
    val: T,
    epoch: usize,
}

///
/// An index referring to an index and epoch in an Arena.
///
#[derive(Copy, Clone, Debug)]
pub struct ArenaIdx{
    index: usize,
    epoch: usize,
}

///
/// An Arena allocator that keeps track of freed cells in a Vec.
///
/// # Example
///
///```rust
///
/// use arena_allocator::arena::*;
///
/// let mut arena = Arena::new();
///
/// let i0 = arena.alloc(0);
/// let i1 = arena.alloc(1);
///
/// assert_eq!(*arena.at(i0).unwrap(), 0);
/// assert_eq!(*arena.at(i1).unwrap(), 1);
///
/// arena.dealloc(i1);
///
/// assert_eq!(arena.at(i1), None);
///
/// let i2 = arena.alloc(2);
///
/// assert_eq!(*arena.at(i2).unwrap(), 2);
/// assert_eq!(arena.at(i1), None);
///
///```
///
#[derive(Debug)]
pub struct Arena<T>{
    cells: Vec<ArenaCell<T>>,
    freed: Vec<usize>,
}

impl<T> Arena<T>{
    pub fn new() -> Self{
        Self{
            cells: Vec::new(),
            freed: Vec::new(),
        }
    }

    // TODO: self in alloc should not have to be mut.
    #[must_use]
    pub fn alloc(&mut self, val: T) -> ArenaIdx{
        match self.freed.last(){
            Some(i) => {
                self.cells[*i].val = val;
                ArenaIdx{
                    index: *i,
                    epoch: self.cells[*i].epoch,
                }
            }
            None => {
                self.cells.push(ArenaCell{
                    epoch: 0,
                    val,
                });
                ArenaIdx{
                    index: self.cells.len() -1,
                    epoch: 0,
                }
            }
        }
    }

    #[inline]
    pub fn dealloc(&mut self, index: ArenaIdx){
        if self.cells[index.index].epoch == index.epoch{
            self.cells[index.index].epoch += 1;
            self.freed.push(index.index);
        }
    }

    #[inline]
    pub fn index(&self, index: ArenaIdx) -> Option<&T>{
        if self.cells[index.index].epoch == index.epoch{
            Some(&self.cells[index.index].val)
        }
        else{
            None
        }
    }
}

#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn test_allocation_deallocation(){
        let mut arena = Arena::new();

        let i0 = arena.alloc(0);
        let i1 = arena.alloc(1);

        assert_eq!(*arena.index(i0).unwrap(), 0);
        assert_eq!(*arena.index(i1).unwrap(), 1);

        arena.dealloc(i1);

        assert_eq!(arena.index(i1), None);

        let i2 = arena.alloc(2);

        assert_eq!(*arena.index(i2).unwrap(), 2);
        assert_eq!(arena.index(i1), None);
    }
}

