
use std::{marker::PhantomData, ops::{Index, IndexMut}};

///
/// Cell of an Arena.
///
#[derive(Debug)]
pub enum ArenaCell<T>{
    Allocated{val: T, generation: usize},
    Freed{next: Option<usize>, generation: usize},
}

///
/// An index referring to an index and epoch in an Arena.
///
#[derive(Copy, Clone, Debug)]
pub struct ArenaIdx<T>{
    index: usize,
    generation: usize,
    _ty: PhantomData<T>,
}

///
/// An Arena allocator that keeps track of freed cells in a Vec.
///
/// # Example
///
///```rust
///
/// use gen_arena::arena::*;
///
/// let mut arena = Arena::new();
///
/// let i0 = arena.insert(0);
/// let i1 = arena.insert(1);
///
/// assert_eq!(*arena.get(i0).unwrap(), 0);
/// assert_eq!(*arena.get(i1).unwrap(), 1);
///
/// arena.remove(i1);
///
/// assert_eq!(arena.get(i1), None);
///
/// let i2 = arena.insert(2);
///
/// assert_eq!(*arena.get(i2).unwrap(), 2);
/// assert_eq!(arena.get(i1), None);
///
///```
///
#[derive(Debug)]
pub struct Arena<T>{
    cells: Vec<ArenaCell<T>>,
    freed: Option<usize>,
    num: usize,
}

impl<T> Arena<T>{

    ///
    /// Creates an empty Arena.
    ///
    ///```rust
    /// use gen_arena::*;
    ///
    /// let arena = Arena::<i32>::new();
    ///```
    ///
    pub fn new() -> Self{
        Self{
            cells: Vec::new(),
            freed: None,
            num: 0,
        }
    }

    ///
    /// Creates an emty Arena with capacity.
    ///
    /// ```rust
    ///
    /// use gen_arena::*;
    ///
    /// let arena = Arena::<i32>::with_capacity(10);
    ///
    /// assert_eq!(arena.capacity(), 10);
    ///
    /// ```
    ///
    pub fn with_capacity(cap: usize) -> Self{
        Self{
            cells: Vec::with_capacity(cap),
            freed: None,
            num: 0,
        }
    }

    ///
    /// Clears the arena and resets the list of Freed cells.
    ///
    /// ```rust
    ///
    /// use gen_arena::*;
    ///
    /// let mut arena = Arena::new();
    ///
    /// let i1 = arena.insert(1);
    /// let i2 = arena.insert(2);
    ///
    /// arena.clear();
    ///
    /// assert_eq!(arena.get(i1), None);
    /// assert_eq!(arena.get(i2), None);
    ///
    /// ```
    ///
    pub fn clear(&mut self){
        let len = self.cells.len();
        for (i, cell) in self.cells.iter_mut().enumerate(){
            match cell{
                ArenaCell::Allocated{val: _, generation} => {
                    *cell = ArenaCell::Freed{
                        generation: *generation + 1,
                        next: if i < len-1 {Some(i+1)} else{None},
                    }
                },
                ArenaCell::Freed{next: _, generation} => {
                    *cell = ArenaCell::Freed{
                        generation: *generation,
                        next: if i < len-1 {Some(i+1)} else{None},
                    }
                }
            }
        }
    }

    ///
    /// Tries to insert into Arena.
    /// Returns val as Err if failed.
    ///
    pub fn try_insert(&mut self, val: T) -> Result<ArenaIdx<T>, T>{
        match self.freed{
            Some(i) => {
                if let ArenaCell::Freed{next, generation} = self.cells[i]{
                    self.freed = next;
                    self.cells[i] = ArenaCell::Allocated{
                        val,
                        generation,
                    };
                    self.num += 1;
                    Ok(ArenaIdx{
                        index: i,
                        generation,
                        _ty: PhantomData,
                    })
                }
                else{
                    //panic!("First freed index pointent to element that was not freed.");
                    Err(val)
                }
            }
            None => {
                self.cells.push(ArenaCell::Allocated{
                    generation: 0,
                    val,
                });
                self.num += 1;
                Ok(ArenaIdx{
                    index: self.cells.len() -1,
                    generation: 0,
                    _ty: PhantomData,
                })
            }
        }
    }

    ///
    /// Inserts a new element into the Arena.
    ///
    /// # Example:
    ///
    /// ```rust 
    /// use gen_arena::*;
    ///
    /// let mut arena = Arena::new();
    ///
    /// let i1 = arena.insert(1);
    /// let i2 = arena.insert(2);
    ///
    /// assert_eq!(*arena.get(i1).unwrap(), 1);
    /// assert_eq!(*arena.get(i2).unwrap(), 2)
    ///
    /// ```
    ///
    #[must_use]
    pub fn insert(&mut self, val: T) -> ArenaIdx<T>{
        match self.try_insert(val){
            Ok(index) => index,
            Err(_val) => panic!("Insertion not successfull."),
        }
    }

    ///
    ///
    ///
    pub fn remove(&mut self, index: ArenaIdx<T>){
        if let ArenaCell::Allocated{val: _, generation} = &self.cells[index.index]{
            self.cells[index.index] = ArenaCell::Freed{
                next: self.freed,
                generation: generation + 1,
            };
            self.num -= 1;
            self.freed = Some(index.index);
        }
    }

    ///
    /// Returns an optional reference to the value at the index.
    ///
    /// ```rust
    /// use gen_arena::*;
    ///
    /// let mut arena = Arena::new();
    ///
    /// let i1 = arena.insert(1);
    ///
    /// assert_eq!(*arena.get(i1).unwrap(), 1);
    ///
    /// arena.remove(i1);
    ///
    /// assert_eq!(arena.get(i1), None);
    ///
    /// ```
    ///
    pub fn get(&self, index: ArenaIdx<T>) -> Option<&T>{
        if let ArenaCell::Allocated{val, generation} = &self.cells[index.index]{
            if *generation == index.generation{
                Some(val)
            }
            else{
                None
            }
        }
        else{
            None
        }
    }

    ///
    /// Returns a mutable optional reference to the value at the index.
    ///
    /// ```rust
    /// use gen_arena::*;
    ///
    /// let mut arena = Arena::new();
    ///
    /// let i1 = arena.insert(1);
    ///
    /// assert_eq!(*arena.get(i1).unwrap(), 1);
    ///
    /// *arena.get_mut(i1).unwrap() = 2;
    ///
    /// assert_eq!(*arena.get(i1).unwrap(), 2);
    ///
    /// arena.remove(i1);
    ///
    /// assert_eq!(arena.get(i1), None);
    ///
    /// ```
    ///
    pub fn get_mut(&mut self, index: ArenaIdx<T>) -> Option<&mut T>{
        if let ArenaCell::Allocated{val, generation} = &mut self.cells[index.index]{
            if *generation == index.generation{
                Some(val)
            }
            else{
                None
            }
        }
        else{
            None
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize{
        self.cells.capacity()
    }

    #[inline]
    pub fn num(&self) -> usize{
        self.num
    }
}

impl<T> Index<ArenaIdx<T>> for Arena<T>{
    type Output = T;

    fn index(&self, index: ArenaIdx<T>) -> &Self::Output {
        self.get(index).expect("There is no element at this index with that generation.")
    }
}

impl<T> IndexMut<ArenaIdx<T>> for Arena<T>{
    fn index_mut(&mut self, index: ArenaIdx<T>) -> &mut Self::Output {
        self.get_mut(index).expect("There is no element at this index with that generation.")
    }
}

#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn test_allocation_deallocation(){
        let mut arena = Arena::new();

        let i0 = arena.insert(0);
        let i1 = arena.insert(1);

        assert_eq!(*arena.get(i0).unwrap(), 0);
        assert_eq!(*arena.get(i1).unwrap(), 1);

        arena.remove(i1);

        assert_eq!(arena.get(i1), None);

        let i2 = arena.insert(2);

        assert_eq!(*arena.get(i2).unwrap(), 2);
        assert_eq!(arena.get(i1), None);
    }
}

