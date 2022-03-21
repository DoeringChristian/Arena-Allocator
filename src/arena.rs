
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
#[derive(Debug)]
pub struct ArenaIdx<T>{
    index: usize,
    generation: usize,
    _ty: PhantomData<T>,
}

impl<T> ArenaIdx<T>{
    fn new(index: usize, generation: usize) -> Self{
        Self{
            index,
            generation,
            _ty: PhantomData,
        }
    }
}

// Have to implement copy and clone myselfe because of generic.
impl<T> Clone for ArenaIdx<T>{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ArenaIdx<T>{}

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
    /// Removes the cell from the arena and increaces its generation.
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
    /// Get N optional references to N indices in the arena.
    ///
    /// ```rust
    /// use gen_arena::*;
    ///
    /// let mut arena = Arena::new();
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
    pub fn getn<const N: usize>(&self, indices: [ArenaIdx<T>; N]) -> [Option<&T>; N]{
        let mut ret = [None; N];

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

    ///
    /// Returns mutable optional references to two distinct values.
    /// Indices have to be different.
    ///
    ///```rust
    /// use gen_arena::*;
    ///
    /// let mut arena = Arena::new();
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
    pub fn get2_mut(&mut self, indices: (ArenaIdx<T>, ArenaIdx<T>)) -> (Option<&mut T>, Option<&mut T>){
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
    /// Retunrs multiple references to elements in diffrent cells.
    /// Indices have to be seperate else the function returns None.
    ///
    pub fn getn_mut<const N: usize>(&mut self, indices: [ArenaIdx<T>; N]) -> Option<[&mut T; N]>{
        unimplemented!();
        let tmp: Option<usize> = None;

        for index in indices{
            if let Some(tmp) = tmp{
                if tmp == index.index{
                    return None;
                }
            }
        }

        unsafe{
            let mut retp: [*mut T; N] = [std::ptr::null_mut(); N];
            for (i, index) in indices.iter().enumerate(){
                match &mut self.cells[index.index]{
                    ArenaCell::Allocated{val, generation} if index.generation == *generation =>{
                        retp[i] = val as *mut T;
                    }
                    _ => {return None},
                }
            }
            let mut ret: [&mut T; N];
            for (i, p) in retp.iter().enumerate(){
            }
        }

        todo!()
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize){
        self.cells.reserve(additional)
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

