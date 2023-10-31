use crate::ring_buffer::RingBuffer;

#[test]
fn test_ring_buffer() {
    let data = vec![0, 1, 2, 3, 4, 5, 6, 7];
    let mut ring_buffer = RingBuffer::new(data.clone());

    ring_buffer.rotate_left(3);
    let mut data_clone = data.iter().map(|i| i).collect::<Vec<_>>();
    data_clone.rotate_left(3);

    let ring_buffer_ordered = ring_buffer.index_range(0..-1);

    assert_eq!(ring_buffer_ordered, data_clone);
}
