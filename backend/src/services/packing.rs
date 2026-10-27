//! Which shipping rates (packages) can carry a cart: by weight and by size.
//!
//! The parcel is estimated from the items' dimensions: either all items stacked on their flattest
//! side, or — if each item fits on its own — by volume with some room left for packing. Items without
//! dimensions only count by weight; rates without maximum dimensions accept any size.

/// Dimensions in millimetres, longest first.
pub type Dims = [i32; 3];

pub fn sorted_dims(l: Option<i32>, w: Option<i32>, h: Option<i32>) -> Option<Dims> {
    let (l, w, h) = (l.unwrap_or(0), w.unwrap_or(0), h.unwrap_or(0));
    if l <= 0 || w <= 0 || h <= 0 {
        return None;
    }
    let mut d = [l, w, h];
    d.sort_unstable_by(|a, b| b.cmp(a));
    Some(d)
}

/// The items of a cart that need shipping.
#[derive(Debug, Default, Clone)]
pub struct Parcel {
    pub weight_g: i32,
    /// Items with known dimensions: (dimensions, quantity)
    pub items: Vec<(Dims, i32)>,
}

impl Parcel {
    pub fn add(&mut self, weight_g: i32, dims: Option<Dims>, qty: i32) {
        self.weight_g = self.weight_g.saturating_add(weight_g.max(0).saturating_mul(qty));
        if let Some(d) = dims {
            self.items.push((d, qty));
        }
    }

    /// Outer size when everything is stacked on the flattest side.
    pub fn stacked(&self) -> Option<Dims> {
        if self.items.is_empty() {
            return None;
        }
        let l = self.items.iter().map(|(d, _)| d[0]).max().unwrap_or(0);
        let w = self.items.iter().map(|(d, _)| d[1]).max().unwrap_or(0);
        let h: i64 = self.items.iter().map(|(d, q)| d[2] as i64 * *q as i64).sum();
        let mut d = [l, w, h.min(i32::MAX as i64) as i32];
        d.sort_unstable_by(|a, b| b.cmp(a));
        Some(d)
    }

    fn volume(&self) -> f64 {
        self.items.iter().map(|(d, q)| d[0] as f64 * d[1] as f64 * d[2] as f64 * *q as f64).sum()
    }

    pub fn fits_size(&self, max: Option<Dims>) -> bool {
        let Some(box_dims) = max else { return true };
        let Some(stacked) = self.stacked() else { return true };
        let inside = |d: &Dims| d.iter().zip(box_dims.iter()).all(|(a, b)| a <= b);
        if inside(&stacked) {
            return true;
        }
        // Many small items: each must fit, and their volume leaves 20 % room for packing
        let box_volume = box_dims[0] as f64 * box_dims[1] as f64 * box_dims[2] as f64;
        self.items.iter().all(|(d, _)| inside(d)) && self.volume() <= box_volume * 0.8
    }

    pub fn fits_weight(&self, min_g: i32, max_g: i32) -> bool {
        self.weight_g >= min_g && (max_g <= 0 || self.weight_g <= max_g)
    }

    pub fn fits(&self, rate: &RateLimits) -> bool {
        self.fits_weight(rate.min_weight_g, rate.max_weight_g) && self.fits_size(rate.max_dims)
    }

    pub fn describe(&self) -> String {
        let weight = format!("{:.2} kg", self.weight_g as f64 / 1000.0);
        match self.stacked() {
            Some(d) => format!("{}, about {} × {} × {} cm", weight, cm(d[0]), cm(d[1]), cm(d[2])),
            None => weight,
        }
    }
}

fn cm(mm: i32) -> String {
    let v = mm as f64 / 10.0;
    if v.fract() == 0.0 { format!("{:.0}", v) } else { format!("{:.1}", v) }
}

#[derive(Debug, Clone)]
pub struct RateLimits {
    pub min_weight_g: i32,
    pub max_weight_g: i32,
    /// Maximum package size; `None` = no size limit
    pub max_dims: Option<Dims>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rate(max_w: i32, dims: Option<(i32, i32, i32)>) -> RateLimits {
        RateLimits { min_weight_g: 0, max_weight_g: max_w, max_dims: dims.and_then(|(l, w, h)| sorted_dims(Some(l), Some(w), Some(h))) }
    }

    #[test]
    fn dimensions_are_sorted_and_need_all_three() {
        assert_eq!(sorted_dims(Some(100), Some(300), Some(200)), Some([300, 200, 100]));
        assert_eq!(sorted_dims(Some(100), None, Some(200)), None);
        assert_eq!(sorted_dims(Some(0), Some(1), Some(1)), None);
    }

    #[test]
    fn picks_packages_by_weight_and_size() {
        // Letter-size package 35 × 25 × 3 cm, up to 1 kg; parcel 60 × 30 × 15 cm up to 5 kg; no-limit freight
        let letter = rate(1000, Some((350, 250, 30)));
        let parcel = rate(5000, Some((600, 300, 150)));
        let freight = rate(0, None);

        let mut book = Parcel::default();
        book.add(400, sorted_dims(Some(240), Some(170), Some(20)), 1);
        assert!(book.fits(&letter) && book.fits(&parcel) && book.fits(&freight));

        // Two books fit side by side (34 × 24 cm); three are too thick stacked and too much volume
        let mut two_books = Parcel::default();
        two_books.add(400, sorted_dims(Some(240), Some(170), Some(20)), 2);
        assert!(two_books.fits(&letter));
        let mut three_books = Parcel::default();
        three_books.add(300, sorted_dims(Some(240), Some(170), Some(20)), 3);
        assert!(!three_books.fits(&letter) && three_books.fits(&parcel));

        // A 70 cm long item never fits the 60 cm parcel, even though it is light
        let mut long = Parcel::default();
        long.add(800, sorted_dims(Some(700), Some(100), Some(100)), 1);
        assert!(!long.fits(&parcel) && long.fits(&freight));

        // Too heavy for the parcel
        let mut heavy = Parcel::default();
        heavy.add(6000, sorted_dims(Some(300), Some(200), Some(100)), 1);
        assert!(!heavy.fits(&parcel));
    }

    #[test]
    fn many_small_items_fit_by_volume() {
        // 30 cubes of 10 cm: stacked 3 m high, but 30 litres fit a 60 × 30 × 30 cm box (54 l)
        let mut cubes = Parcel::default();
        cubes.add(100, sorted_dims(Some(100), Some(100), Some(100)), 30);
        assert!(cubes.fits_size(sorted_dims(Some(600), Some(300), Some(300))));
        assert!(!cubes.fits_size(sorted_dims(Some(300), Some(300), Some(300))));
    }

    #[test]
    fn items_without_size_only_count_by_weight() {
        let mut p = Parcel::default();
        p.add(500, None, 2);
        assert_eq!(p.weight_g, 1000);
        assert!(p.fits(&rate(1000, Some((100, 100, 10)))));
        assert_eq!(p.describe(), "1.00 kg");
    }
}
