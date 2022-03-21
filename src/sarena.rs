
use std::marker::PhantomData;

use crate::*;

///
/// An index referring to an index and epoch in an Arena.
///
#[derive(Debug, PartialEq, Eq)]
pub struct SArenaIdx<T>{
    index: usize,
    generation: usize,
    _ty: PhantomData<T>,
}

impl<T> SArenaIdx<T>{
    pub fn new(index: usize, generation: usize) -> Self{
        Self{
            index,
            generation,
            _ty: PhantomData,
        }
    }

    #[inline]
    pub fn index(&self) -> usize{
        self.index
    }

    #[inline]
    pub fn gen(&self) -> usize{
        self.generation
    }
}

impl<T> Clone for SArenaIdx<T>{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for SArenaIdx<T>{}

pub struct SArena<T, const N: usize>{
    cells: [ArenaCell<T>; N],
    freed: Option<usize>,
    num: usize,
}

impl<T, const N: usize> SArena<T, N>{
    ///
    /// Creates a new empty SArena.
    ///
    ///```rust
    /// use gen_arena::*;
    ///
    /// let arena = SArena::<i32, 100>::new();
    ///```
    ///
    pub fn new() -> Self{
        let mut i = 0;
        let cells: [ArenaCell<T>; N] = [(); N].map(|()|{
            let ret = {
                if i < N -1{
                    ArenaCell::Freed{next: Some(i +1), generation: 0}
                }
                else{
                    ArenaCell::Freed{next: None, generation: 0}
                }
            };
            i += 1;
            ret
        });
        
        Self{
            cells,
            freed: Some(0),
            num: 0,
        }
    }

    ///
    /// Tries to insert a value into the Arena.
    /// Unlike Arena::try_insert this does not need a mut ref 
    /// because the array stays in the same place all the time.
    ///
    #[must_use]
    pub fn try_insert(&self, val: T) -> Result<SArenaIdx<T>, T>{

        // SAFETY: 
        // - Insertion abborts if cell is iccupied hence only freed cells are affected.
        // - The memory location of cells does not change on insertion unlike Vec.
        unsafe{
            let selfp = (self as *const Self) as *mut Self;
            match self.freed{
                Some(i) => {
                    if let ArenaCell::Freed{next, generation} = self.cells[i]{
                        (*selfp).freed = next;
                        (*selfp).cells[i] = ArenaCell::Allocated{
                            val,
                            generation,
                        };
                        (*selfp).num += 1;
                        Ok(SArenaIdx::new(i, generation))
                    }
                    else{
                        Err(val)
                    }
                }
                None => Err(val)
            }
        }
    }

    ///
    /// Inserts a new element into the Arena.
    /// Panics if it is full.
    ///
    /// # Example:
    ///
    /// ```rust 
    /// use gen_arena::*;
    ///
    /// let arena = SArena::<_, 100>::new();
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
    pub fn insert(&self, val: T) -> SArenaIdx<T>{
        match self.try_insert(val){
            Ok(index) => index,
            Err(_val) => panic!("Insertion not successfull."),
        }
    }

    ///
    /// Removes the cell from the arena and increaces its generation.
    ///
    pub fn remove(&mut self, index: SArenaIdx<T>){
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
    /// Gets the Generation for a given index.
    ///
    pub fn gen(&self, index: usize) -> usize{
        match self.cells[index]{
            ArenaCell::Freed{generation, ..} => generation,
            ArenaCell::Allocated{generation, ..} => generation,
        }
    }

    ///
    /// Returns an optional reference to the value at the index.
    ///
    /// ```rust
    /// use gen_arena::*;
    ///
    /// let mut arena = SArena::<_, 100>::new();
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
    pub fn get(&self, index: SArenaIdx<T>) -> Option<&T>{
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
    /// Returns an optional reference to a cell with any generation.
    ///
    /// Get N optional references to N indices in the arena.
    ///
    /// ```rust
    /// use gen_arena::*;
    ///
    /// let mut arena = SArena::<_, 100>::new();
    ///
    /// let i1 = arena.insert(1);
    /// let i2 = arena.insert(2);
    ///
    /// let res = arena.getn([i1, i2]);
    ///
    /// assert_eq!(*res[0].unwrap(), 1);
    /// assert_eq!(*res[1].unwrap(), 2);
    ///
    /// ```
    ///
    ///
    pub fn get_any(&self, index: usize) -> Option<&T>{
        if let ArenaCell::Allocated{val, generation: _} = &self.cells[index]{
            Some(val)
        }
        else{
            None
        }
    }

    ///
    /// Get N optional references to N indices in the arena.
    ///
    /// ```rust
    /// use gen_arena::*;
    ///
    /// let mut arena = SArena::<_, 100>::new();
    ///
    /// let i1 = arena.insert(1);
    /// let i2 = arena.insert(2);
    ///
    /// let res = arena.getn([i1, i2]);
    ///
    /// assert_eq!(*res[0].unwrap(), 1);
    /// assert_eq!(*res[1].unwrap(), 2);
    ///
    /// ```
    ///
    pub fn getn<const M: usize>(&self, indices: [SArenaIdx<T>; M]) -> [Option<&T>; M]{
        let mut ret = [None; M];

        for (i, index) in indices.iter().enumerate(){
            ret[i] = self.get(*index);
        }
        ret
    }

    ///
    /// Returns a mutable optional reference to the value at the index.
    ///
    /// ```rust
    /// use gen_arena::*;
    ///
    /// let mut arena = SArena::<_, 100>::new();
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
    pub fn get_mut(&mut self, index: SArenaIdx<T>) -> Option<&mut T>{
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

    pub fn get_any_mut(&mut self, index: usize) -> Option<&mut T>{
        if let ArenaCell::Allocated{val, generation: _} = &mut self.cells[index]{
            Some(val)
        }
        else{
            None
        }
    }

    ///
    /// Returns mutable optional references to two distinct values.
    /// Indices have to be different.
    ///
    ///```rust
    /// use gen_arena::*;
    ///
    /// let mut arena = SArena::<_, 100>::new();
    ///
    /// let i1 = arena.insert(1);
    /// let i2 = arena.insert(2);
    ///
    /// let (c1, c2) = arena.get2_mut((i1, i2));
    ///
    /// *c1.unwrap() = 3;
    /// *c2.unwrap() = 4;
    ///
    /// assert_eq!(*arena.get(i1).unwrap(), 3);
    /// assert_eq!(*arena.get(i2).unwrap(), 4);
    ///
    ///```
    ///
    pub fn get2_mut(&mut self, indices: (SArenaIdx<T>, SArenaIdx<T>)) -> (Option<&mut T>, Option<&mut T>){
        if indices.0.index == indices.1.index{
            if indices.0.generation == indices.1.generation{
                panic!("Cannot take 2 mutable references to a value at the same index.")
            }

            if indices.0.generation > indices.1.generation{
                return (self.get_mut(indices.0), None);
            }
            else{
                return (None, self.get_mut(indices.1));
            }
        }

        if indices.0.index >= self.cells.len(){
            return (None, self.get_mut(indices.1));
        }
        if indices.1.index >= self.cells.len(){
            return (self.get_mut(indices.0), None);
        }

        let (cell0, cell1) = {
            let split = self.cells.split_at_mut(indices.0.index.max(indices.1.index));
            if indices.0.index < indices.1.index{
                (&mut split.0[indices.0.index], &mut split.1[0])
            }
            else{
                (&mut split.1[0], &mut split.0[indices.1.index])
            }
        };

        let cell0 = match cell0{
            ArenaCell::Allocated{val, generation} => {
                if indices.0.generation == *generation{
                    Some(val)
                }
                else{
                    None
                }
            },
            _ => None
        };
        let cell1 = match cell1{
            ArenaCell::Allocated{val, generation} => {
                if indices.1.generation == *generation{
                    Some(val)
                }
                else{
                    None
                }
            },
            _ => None
        };

        (cell0, cell1)
    }

    ///
    /// Returns iterator over all Allocated cells.
    ///
    /// ```rust
    /// use gen_arena::*;
    /// let mut arena = SArena::<_, 100>::new();
    ///
    /// let i1 = arena.insert(1);
    /// let i2 = arena.insert(1);
    ///
    /// for val in arena.iter(){
    ///     assert_eq!(*val, 1);
    /// }
    ///
    /// ```
    ///
    #[inline]
    pub fn iter(&self) -> Iter<T>{
        Iter{
            iter: self.enumerate()
        }
    }

    ///
    /// Returns mutable iterator over all Allocated cells.
    ///
    /// ```rust
    /// use gen_arena::*;
    /// let mut arena = SArena::<_, 100>::new();
    ///
    /// let i1 = arena.insert(1);
    /// let i2 = arena.insert(2);
    ///
    /// for val in arena.iter_mut(){
    ///     *val = 0;
    /// }
    ///
    /// assert_eq!(*arena.get(i1).unwrap(), 0);
    /// assert_eq!(*arena.get(i2).unwrap(), 0);
    ///
    /// ```
    ///
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<T>{
        IterMut{
            iter: self.enumerate_mut()
        }
    }

    ///
    /// Returns an iterator over the Allocated cells with index.
    ///
    /// TODO: either add new iterator type for SArena or use ArenaIdx for SArena.
    /// ```rust, ignore
    /// use gen_arena::*;
    /// let mut arena = SArena::<_, 100>::new();
    ///
    /// let i1 = arena.insert(1);
    /// let i2 = arena.insert(2);
    ///
    /// for (index, val) in arena.enumerate(){
    ///     if index == i1{
    ///         assert_eq!(*val, 1);
    ///     }
    ///     if index == i2{
    ///         assert_eq!(*val, 2);
    ///     }
    /// }
    ///
    /// ```
    ///
    #[inline]
    pub fn enumerate(&self) -> Enumerator<T>{
        Enumerator{
            iter: self.cells.iter().enumerate(),
        }
    }

    ///
    /// Returns an mutable iterator over the Allocated cells with indices.
    ///
    /// ```rust
    /// use gen_arena::*;
    /// let mut arena = SArena::<_, 100>::new();
    ///
    /// let i1 = arena.insert(1);
    /// let i2 = arena.insert(2);
    /// 
    /// for (index, val) in arena.enumerate_mut(){
    ///     *val = index.index();
    /// }
    ///
    /// assert_eq!(*arena.get(i1).unwrap(), 0);
    /// assert_eq!(*arena.get(i2).unwrap(), 1);
    ///
    /// ```
    ///
    #[inline]
    pub fn enumerate_mut(&mut self) -> EnumeratorMut<T>{
        EnumeratorMut{
            iter: self.cells.iter_mut().enumerate(),
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize{
        N
    }

    #[inline]
    pub fn num(&self) -> usize{
        self.num
    }
}
