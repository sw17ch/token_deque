use proptest::prelude::*;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use token_deque::Deque;

proptest! {
    #[test]
    fn random_push_and_pop(
        pushes in proptest::collection::vec(any::<bool>(), 0..64),
        pops in proptest::collection::vec(any::<bool>(), 0..64)
    ) {
        let mut l: Deque<usize> = Deque::new();

        let len = pushes.len();

        for (p,v) in pushes.into_iter().zip((0..len).into_iter()) {
            if p {
                l.push_front(v);
            } else {
                l.push_back(v);
            }
        }

        for p in pops {
            if p {
                l.pop_front();
            } else {
                l.pop_back();
            }
        }
    }
}
proptest! {
    #[test]
    fn random_interleaved_push_and_pop(
        action in proptest::collection::vec(any::<usize>(), 0..64)
    ){
        let mut l: Deque<usize> = Deque::new();

        for a in action {
            match a & 0x03 {
                0x00 => {
                    l.push_front(a);
                },
                0x01 => {
                    l.push_back(a);
                },
                0x02 => {
                    l.pop_front();
                },
                0x03 => {
                    l.pop_back();
                },
                _ => unreachable!(),
            }
        }
    }
}

proptest! {
    #[test]
    fn random_remove(
        seed in any::<u64>(),
        pushes in proptest::collection::vec(any::<usize>(), 0..64),
    ) {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut list = Deque::new();
        let mut tokens = Vec::new();

        for p in pushes {
            let tok = list.push_back(p);
            tokens.push((tok,p));
        }

        tokens.shuffle(&mut rng);

        for (t, p) in tokens {
            assert_eq!(Some(p), list.remove(&t));
        }

        assert!(list.is_empty());
    }
}
