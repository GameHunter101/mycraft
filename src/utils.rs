use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
    sync::{Arc, Mutex},
};

use gamezap::model::{Mesh, MeshManager};

pub trait MeshTools {
    fn create_mesh(
        &self,
        device: Arc<wgpu::Device>,
        mesh_manager: Arc<Mutex<MeshManager>>,
    ) -> Arc<Mesh>;
}

pub struct RingBuffer<T> {
    pub buffer: Box<[T]>,
    ring_start: i32,
}

#[allow(unused)]
impl<T> RingBuffer<T> {
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
}

impl<T: Debug> Debug for RingBuffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ordered = &self.buffer[..];
        write!(f, "{:?}", ordered)
    }
}

impl<T> Index<i32> for RingBuffer<T> {
    fn index(&self, index: i32) -> &Self::Output {
        &self.buffer[self.recalculate_index(index)]
    }
    type Output = T;
}

impl<T> IndexMut<i32> for RingBuffer<T> {
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

impl<T> Index<std::ops::Range<i32>> for RingBuffer<T> {
    fn index(&self, index: std::ops::Range<i32>) -> &Self::Output {
        let start = self.recalculate_index(index.start);
        let end = self.recalculate_index(index.end);
        &self.buffer[start..end]
    }
    type Output = [T];
}

impl<T> Index<std::ops::RangeFrom<i32>> for RingBuffer<T> {
    fn index(&self, index: std::ops::RangeFrom<i32>) -> &Self::Output {
        let start = self.recalculate_index(index.start);
        &self.buffer[start..]
    }
    type Output = [T];
}

impl<T> Index<std::ops::RangeFull> for RingBuffer<T> {
    fn index(&self, _index: std::ops::RangeFull) -> &Self::Output {
        &self.buffer[0..]
    }
    type Output = [T];
}

/* impl<T> IntoIterator for RingBuffer<T> {
    fn into_iter(self) -> Self::IntoIter {
        let buff = self[..].iter().map(|&i| i).collect::<Vec<_>>();
        buff.into_iter()
        // self.buffer[..].to_vec().into_iter()
    }
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
} */
