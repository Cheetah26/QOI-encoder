pub fn encode<T: Into<u32>>(width: T, height: T, channels: u8, data: &mut Vec<u8>) -> Vec<u8> {
  let mut encoded = Vec::new();

  // Header
  encoded.append(&mut [0x71, 0x6f, 0x69, 0x66].to_vec());
  encoded.append(&mut width.into().to_be_bytes().to_vec());
  encoded.append(&mut height.into().to_be_bytes().to_vec());
  encoded.append(&mut [channels, 0x0].to_vec());

  let mut screen: [&[u8]; 64] = [&[0; 4]; 64];
  let mut prev: &[u8] = &[0, 0, 0, 255];
  let mut run = 0;

  let mut pixels = data.chunks_exact_mut(channels as usize);

  while let Some(pixel) = pixels.next() {
    let (r, g, b, a) = (
      pixel[0],
      pixel[1],
      pixel[2],
      (if channels == 3 { 255 } else { pixel[3] }),
    );

    // Current run
    if pixel == prev && run < 62 {
      run += 1;
      continue;
    }

    // Ended run
    if run != 0 {
      encoded.push(0xC0 | (run - 1));
      if pixel == prev {
        run = 1;
        continue;
      } else {
        run = 0;
      }
    }

    // Index
    let hash = ((r as u32 * 3 + g as u32 * 5 + b as u32 * 7 + a as u32 * 11) % 64) as usize;
    if screen[hash] == pixel {
      encoded.push(hash as u8);
      prev = pixel;
      continue;
    }

    // Diff
    let diffs: Vec<i8> = pixel
      .iter()
      .take(3)
      .enumerate()
      .map(|(i, v)| (*v as i8).wrapping_sub(prev[i] as i8))
      .collect();

    if diffs.iter().all(|&d| d >= -2 && d <= 1) {
      let mut data = 0;
      diffs
        .iter()
        .for_each(|&d| data = (data << 2) | (d + 2) as u8);
      encoded.push(0x40 | data);
      screen[hash] = pixel;
      prev = pixel;
      continue;
    }

    // Luma
    let (dr, dg, db) = (diffs[0], diffs[1], diffs[2]);
    if dg >= -32 && dg <= 31 {
      let dr = dr.wrapping_sub(dg);
      let db = db.wrapping_sub(dg);
      if (dr >= -8 && dr <= 7) && (db >= -8 && db <= 7) {
        encoded.push(0x80 | ((dg + 32) as u8));
        encoded.push(((dr + 8) as u8) << 4 | (db + 8) as u8);
        screen[hash] = pixel;
        prev = pixel;
        continue;
      }
    }

    // RGB
    if channels == 3 {
      encoded.append(&mut [0xFE, r, g, b].to_vec());
    } else {
      encoded.append(&mut [0xFF, r, g, b, a].to_vec());
    }
    screen[hash] = pixel;
    prev = pixel;
  }

  // Complete run
  if run != 0 {
    encoded.push(0xC0 | (run - 1));
  }

  // EOS
  encoded.append(&mut [0; 7].to_vec());
  encoded.push(0x01);

  encoded
}
