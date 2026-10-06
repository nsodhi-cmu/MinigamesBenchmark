use crate::rng::Rng;

#[test]
fn a_bounded_draw_stays_inside_its_bound() {
    let mut rng = Rng::new(3);
    assert_eq!(rng.below(0), 0, "a bound of nothing has to answer rather than divide by it");
    assert!((0..4096).all(|_| (0.0..1.0).contains(&rng.unit())));
}

#[test]
fn what_is_taken_is_all_different_and_all_from_what_was_offered() {
    let mut rng = Rng::new(4);
    let offered: Vec<u32> = (10..40).collect();
    for count in [0usize, 1, 7, 30] {
        let taken = rng.take(&offered, count);
        assert_eq!(taken.len(), count);
        for (slot, item) in taken.iter().enumerate() {
            assert!(offered.contains(item), "{item} was not offered");
            assert!(!taken[..slot].contains(item), "{item} was taken twice");
        }
    }
}

#[test]
fn taking_more_than_is_offered_takes_all_of_it_rather_than_looping() {
    let mut rng = Rng::new(9);
    assert_eq!(rng.take(&[1u8, 2, 3], 99).len(), 3);
    assert!(rng.take::<u8>(&[], 4).is_empty());
}
