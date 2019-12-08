#[cfg(not(feature = "decode"))]
use std::collections::HashMap;
use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Color {
    Black,
    White,
    Transparent,
}

impl From<char> for Color {
    fn from(c: char) -> Self {
        match c {
            '0' => Color::Black,
            '1' => Color::White,
            '2' => Color::Transparent,
            c => panic!("Invalid color {}", c),
        }
    }
}

type ImageData = Vec<Color>;
struct Layer<'a> {
    rows: Vec<&'a [Color]>,
}
type Layers<'a> = Vec<Layer<'a>>;

#[cfg(not(feature = "decode"))]
impl Layer<'_> {
    fn frequencies(&self) -> HashMap<Color, usize> {
        let mut frequencies = HashMap::new();
        for row in &self.rows {
            for cell in *row {
                *frequencies.entry(*cell).or_insert(0) += 1;
            }
        }
        frequencies
    }
}

#[cfg(feature = "decode")]
struct Image {
    rows: Vec<Vec<Color>>,
}

#[cfg(feature = "decode")]
impl Image {
    fn new() -> Self {
        Image { rows: Vec::new() }
    }

    fn apply(&mut self, layer: &Layer) {
        for row_index in 0..layer.rows.len() {
            let layer_row = layer.rows[row_index];
            if row_index >= self.rows.len() {
                self.rows.push(Vec::new());
            }
            let row = self.rows.get_mut(row_index).unwrap();
            for cell_index in 0..layer_row.len() {
                use self::Color::*;
                let cell = row.get(cell_index);
                let new_color = match (cell, layer_row[cell_index]) {
                    (None, color) => color,
                    (Some(Black), _) => Black,
                    (Some(White), _) => White,
                    (Some(Transparent), color) => color,
                };
                let cell = row.get_mut(cell_index);
                match cell {
                    Some(c) => *c = new_color,
                    None => row.push(new_color),
                }
            }
        }
    }
}

fn build_layers<'a>(wide: usize, tall: usize, image_data: &'a ImageData) -> Layers<'a> {
    let mut layers = Layers::new();
    let layer_size = wide * tall;
    let layer_count = image_data.len() / layer_size;
    for layer_index in 0..layer_count {
        let rows = (0..tall)
            .map(|i| layer_index * layer_size + i * wide)
            .map(|start_index| &image_data[start_index..start_index + wide])
            .collect();
        layers.push(Layer { rows });
    }
    layers
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        panic!("Four arguments needed, received {:#?}", args);
    }
    let wide = args[1].parse().unwrap();
    let tall = args[2].parse().unwrap();
    let file = File::open(&args[3])?;
    let mut reader = BufReader::new(file);
    let mut image_data = String::new();
    reader.read_line(&mut image_data)?;
    let image_data = image_data.trim().chars().map(Color::from).collect();
    let layers = build_layers(wide, tall, &image_data);
    #[cfg(not(feature = "decode"))]
    {
        let min_layer = layers
            .into_iter()
            .min_by_key(|layer| *layer.frequencies().get(&Color::Black).unwrap())
            .unwrap();
        let frequencies = min_layer.frequencies();
        let score = *frequencies.get(&Color::White).unwrap()
            * *frequencies.get(&Color::Transparent).unwrap();
        println!("Final score is {}", score);
    }
    #[cfg(feature = "decode")]
    {
        let mut image = Image::new();
        for layer in &layers {
            image.apply(layer);
        }
        for row in image.rows {
            for cell in row {
                use self::Color::*;
                let c = match cell {
                    White => '█',
                    _ => ' ',
                };
                print!("{}", c);
            }
            println!();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use self::Color::*;
    use super::*;

    #[test]
    fn get_layers() {
        let data = vec![
            White,
            Transparent,
            Black,
            Black,
            Black,
            White,
            White,
            White,
            Transparent,
            Black,
            White,
            Transparent,
        ];
        let layers = build_layers(3, 2, &data);
        assert_eq!(2, layers.len());

        let layer = &layers[0];
        assert_eq!([White, Transparent, Black], layer.rows[0]);
        assert_eq!([Black, Black, White], layer.rows[1]);

        let layer = &layers[1];
        assert_eq!([White, White, Transparent], layer.rows[0]);
        assert_eq!([Black, White, Transparent], layer.rows[1]);
    }

    #[cfg(not(feature = "decode"))]
    #[test]
    fn frequencies() {
        let data = vec![Black, Black, White, Black];
        let layer = Layer {
            rows: vec![&data[0..2], &data[2..4]],
        };
        let frequencies = layer.frequencies();
        assert_eq!(3, *frequencies.get(&Black).unwrap());
        assert_eq!(1, *frequencies.get(&White).unwrap());
    }

    #[cfg(feature = "decode")]
    #[test]
    fn decode() {
        let data = vec![
            Black,
            Transparent,
            Transparent,
            Transparent,
            White,
            White,
            Transparent,
            Transparent,
            Transparent,
            Transparent,
            White,
            Transparent,
            Black,
            Black,
            Black,
            Black,
        ];
        let layer1 = Layer {
            rows: vec![&data[0..2], &data[2..4]],
        };
        let layer2 = Layer {
            rows: vec![&data[4..6], &data[6..8]],
        };
        let layer3 = Layer {
            rows: vec![&data[8..10], &data[10..12]],
        };
        let layer4 = Layer {
            rows: vec![&data[12..14], &data[14..16]],
        };
        let mut image = Image::new();
        image.apply(&layer1);
        image.apply(&layer2);
        image.apply(&layer3);
        image.apply(&layer4);
        assert_eq!(Black, image.rows[0][0]);
    }
}
