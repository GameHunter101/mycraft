use std::{
    borrow::BorrowMut,
    fmt::Debug,
    ops::{Index, IndexMut, Range},
};

pub struct RingBuffer<T> {
    pub buffer: Box<[T]>,
    ring_start: i32,
}

#[allow(unused)]
impl<T: Debug> RingBuffer<T> {
    pub fn new(data: Vec<T>) -> Self {
        RingBuffer {
            buffer: data.into_boxed_slice(),
            ring_start: 0,
        }
    }
    pub fn rotate_left(&mut self, amount: u32) {
        self.ring_start += amount as i32;
    }

    pub fn rotate_right(&mut self, amount: u32) {
        self.ring_start -= amount as i32;
    }

    pub fn replace_first(&mut self, element: T) {
        self[0] = element;
    }

    pub fn replace_last(&mut self, element: T) {
        self[-1] = element;
    }

    fn recalculate_index(&self, index: i32) -> usize {
        let mut index = ((index + self.ring_start) % self.buffer.len() as i32);
        if index < 0 {
            index = self.buffer.len() as i32 + index;
        }
        index as usize
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn index_range(&self, range: Range<i32>) -> Vec<&T> {
        let start = self.recalculate_index(range.start);
        let end = self.recalculate_index(range.end);
        let mut returned_data: Vec<&T> = vec![]; //Vec::with_capacity(capacity);

        if start > end {
            let (end_chain, start_chain) = self.buffer.split_at(start);
            returned_data = start_chain.into_iter().chain(end_chain).collect();
            return returned_data;
        }
        for element in &self.buffer[start..=end] {
            returned_data.push(element);
        }
        returned_data
    }

    pub fn index_range_mut(&mut self, range: Range<i32>) -> Vec<&mut T> {
        let start = self.recalculate_index(range.start);
        let end = self.recalculate_index(range.end);
        let mut returned_data: Vec<&mut T> = vec![];

        if start > end {
            let (end_chain, start_chain) = self.buffer.split_at_mut(start);
            returned_data = start_chain.iter_mut().chain(end_chain).collect();
            return returned_data;
        }

        for element in self.buffer[start..=end].iter_mut() {
            returned_data.push(element);
        }
        returned_data
    }
}

impl<T: Debug> Debug for RingBuffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ordered = &self.index_range(0..-1);
        write!(f, "{:?}", ordered)
    }
}

impl<T: Debug> Index<i32> for RingBuffer<T> {
    fn index(&self, index: i32) -> &Self::Output {
        &self.buffer[self.recalculate_index(index)]
    }
    type Output = T;
}

impl<T: Debug> IndexMut<i32> for RingBuffer<T> {
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        &mut self.buffer[self.recalculate_index(index)]
    }
}

impl<T> FromIterator<T> for RingBuffer<T> {
    fn from_iter<B: IntoIterator<Item = T>>(iter: B) -> Self {
        RingBuffer {
            buffer: iter.into_iter().collect::<Vec<_>>().into_boxed_slice(),
            ring_start: 0,
        }
    }
}

impl<T: Clone> From<&Vec<T>> for RingBuffer<T> {
    fn from(value: &Vec<T>) -> Self {
        RingBuffer {
            buffer: value.clone().into_boxed_slice(),
            ring_start: 0,
        }
    }
}

impl<'a, T: Debug> IntoIterator for &'a RingBuffer<T> {
    type Item = &'a T;
    type IntoIter = std::vec::IntoIter<&'a T>;
    fn into_iter(self) -> Self::IntoIter {
        let slice = self.index_range(0..-1);
        slice.into_iter()
    }
}

impl<'a, T: Debug> IntoIterator for &'a mut RingBuffer<T> {
    type Item = &'a mut T;
    type IntoIter = std::vec::IntoIter<&'a mut T>;
    fn into_iter(self) -> Self::IntoIter {
        self.index_range_mut(0..-1).into_iter()
    }
}

pub struct RingBuffer2D<T> {
    pub buffer: RingBuffer<RingBuffer<T>>,
    pub horizontal_start: i32,
    pub vertical_start: i32,
}

#[allow(unused)]
impl<T: Clone + Debug> RingBuffer2D<T> {
    pub fn new(data: Vec<Vec<T>>) -> Self {
        RingBuffer2D {
            buffer: data.into_iter().map(|i| RingBuffer::new(i)).collect(),
            horizontal_start: 0,
            vertical_start: 0,
        }
    }
    pub fn rotate_left(&mut self, amount: u32) {
        self.horizontal_start += amount as i32;
        self.buffer.rotate_left(1);
    }

    pub fn rotate_right(&mut self, amount: u32) {
        self.horizontal_start -= amount as i32;
        self.buffer.rotate_right(1);
    }

    pub fn rotate_up(&mut self, amount: u32) {
        self.vertical_start -= amount as i32;
        for vertical_slice in &mut self.buffer {
            vertical_slice.rotate_right(1)
        }
    }

    pub fn rotate_down(&mut self, amount: u32) {
        self.vertical_start += amount as i32;
        for vertical_slice in &mut self.buffer {
            vertical_slice.rotate_left(1)
        }
    }

    /// Replace the first vertical slice
    pub fn replace_first(&mut self, element: RingBuffer<T>) {
        self[0] = element;
    }

    // Replace the last vertical slice
    pub fn replace_last(&mut self, element: RingBuffer<T>) {
        self[-1] = element;
    }

    fn recalculate_index(&self, index: i32) -> usize {
        let mut index = ((index + self.vertical_start) % self.buffer[0].len() as i32);
        if index < 0 {
            index = self.buffer.len() as i32 + index;
        }
        index as usize
    }

    pub fn index_horizontal(&self, index: i32) -> Vec<T> {
        let mut slice: Vec<T> = Vec::with_capacity(self.buffer.len());
        for vertical_slice in self.into_iter() {
            slice.push(vertical_slice[index].clone());
        }
        slice
    }

    pub fn mut_index_horizontal(&mut self, index: i32, data: &[T]) {
        for (i, vertical_slice) in self.buffer.borrow_mut().into_iter().enumerate() {
            vertical_slice[index] = data[i].clone();
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn flatten(&self) -> Vec<T> {
        let mut size = self.into_iter().map(|e| e.len()).sum();
        let mut flattened = Vec::with_capacity(size);
        let mut count = 0;
        for z_slice in self.into_iter() {
            count += 1;
            for e in z_slice.into_iter() {
                flattened.push(e.clone());
            }
        }
        flattened
    }
}

/// Index gives vertical slices
impl<T: Debug> Index<i32> for RingBuffer2D<T> {
    fn index(&self, index: i32) -> &Self::Output {
        &self.buffer[index]
    }
    type Output = RingBuffer<T>;
}

/// Mutate vertical slices
impl<T: Debug> IndexMut<i32> for RingBuffer2D<T> {
    fn index_mut(&mut self, index: i32) -> &mut Self::Output {
        &mut self.buffer[index]
    }
}

impl<'a, T: Debug> IntoIterator for &'a RingBuffer2D<T> {
    type Item = &'a RingBuffer<T>;
    type IntoIter = std::vec::IntoIter<&'a RingBuffer<T>>;
    fn into_iter(self) -> Self::IntoIter {
        self.buffer.into_iter()
    }
}
