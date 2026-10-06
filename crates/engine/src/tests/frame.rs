use crate::frame::{Frame, HEIGHT, Rgb, WIDTH};
use crate::palette;

fn count(frame: &Frame, color: Rgb) -> usize {
    let packed = {
        let mut probe = Frame::new();
        probe.fill(color);
        probe.pixels()[0]
    };
    frame.pixels().iter().filter(|p| **p == packed).count()
}

#[test]
fn a_shape_drawn_off_the_edge_is_clipped_rather_than_wrapped() {
    let mut frame = Frame::new();
    frame.fill(palette::BACKDROP);
    frame.rect(-40.0, -40.0, 60.0, 60.0, palette::GOOD);
    assert_eq!(count(&frame, palette::GOOD), 20 * 20);
    assert_eq!(frame.pixels().len(), (WIDTH * HEIGHT) as usize);
}

#[test]
fn a_filled_square_polygon_covers_what_the_same_rectangle_covers() {
    let mut drawn = Frame::new();
    drawn.poly(&[[100.0, 100.0], [180.0, 100.0], [180.0, 160.0], [100.0, 160.0]], palette::MARK);
    let mut expected = Frame::new();
    expected.rect(100.0, 100.0, 80.0, 60.0, palette::MARK);
    assert_eq!(count(&drawn, palette::MARK), count(&expected, palette::MARK));
}

#[test]
fn a_disc_is_round_rather_than_square() {
    let mut frame = Frame::new();
    frame.disc(320.0, 240.0, 50.0, palette::BAD);
    let area = count(&frame, palette::BAD) as f32;
    assert!((area - std::f32::consts::PI * 2500.0).abs() < 200.0, "area was {area}");
}

#[test]
fn a_frame_encodes_to_a_png_of_its_own_size() {
    let mut frame = Frame::new();
    frame.fill(palette::PANEL);
    let png = frame.to_png();
    assert_eq!(&png[1..4], b"PNG");
    let decoder = png::Decoder::new(std::io::Cursor::new(&png));
    let reader = decoder.read_info().expect("the header is readable");
    assert_eq!(reader.info().size(), (WIDTH, HEIGHT));
}
