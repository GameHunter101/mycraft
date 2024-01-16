#[allow(unused)]
use crate::ring_buffer::{RingBuffer, RingBuffer2D};

#[test]
fn test_ring_buffer() {
    let mut count = 0;
    let data = (0..5)
        .map(|_| {
            (0..5)
                .map(|_| {
                    count += 1;
                    count
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    count = 0;
    let flattened_data = (0..5)
        .map(|_| {
            (0..5)
                .map(|_| {
                    count += 1;
                    count
                })
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect::<Vec<_>>();

    let buf = RingBuffer2D::new(data);

    let index = (-1, 4);
    let linearized_index = buf.linearize_index(index);
    let recalculated_x = buf.recalculate_index_horizontal(index.0);
    let recalculated_y = buf.recalculate_index_vertical(index.1);
    let recalculated_index = (recalculated_x as i32, recalculated_y as i32);

    for y in &buf {
        for x in y {
            print!("{x} ")
        }
        println!();
    }
    for x in &flattened_data {
        print!("{x} ");
    }
    println!();
    println!("normal: {index:?}, {}", buf[index]);
    println!("recalculated: {recalculated_index:?}, {}", buf[recalculated_index]);
    println!("linearized: {linearized_index}, {}", flattened_data[linearized_index]);
    assert_eq!(buf[index], flattened_data[linearized_index]);
}
