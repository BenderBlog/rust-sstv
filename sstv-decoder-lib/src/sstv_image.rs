// Copyright 2025 BenderBlog Rodriguez and Contributors.
// SPDX-License-Identifier: 0BSD

// use thiserror::Error;
//
// #[derive(Error, Debug)]
// pub enum ImageCreationError {
//     #[error("Invalid image size {0}. Should be {1}")]
//     VectorSize(usize, usize),
// }
//
// /// The image structure is just a one-demension vector of (r,g,b) values.
// /// With width and height info.
// pub struct SSTVImage {
//     width: usize,
//     height: usize,
//     image: Vec<[u8; 3]>,
// }
//
// impl SSTVImage {
//     /// Generate a new SSTVImage.
//     ///
//     /// Will return an error if the size of the picture is not match with your input.
//     pub fn new(
//         width: usize,
//         height: usize,
//         image: Vec<[u8; 3]>,
//     ) -> Result<Self, ImageCreationError> {
//         if image.len() != width * height {
//             return Err(ImageCreationError::VectorSize(image.len(), width * height));
//         }
//
//         Ok(Self {
//             width,
//             height,
//             image,
//         })
//     }
//
//     /// Generate a new SSTVImage from DynamicImage, image feature should be enabled first.
//     #[cfg(feature = "image")]
//     pub fn from_image(image: image::DynamicImage) -> Result<Self, ImageCreationError> {
//         let rgb_image = image.to_rgb8().to_vec();
//         let mut i: usize = 0;
//         let mut result: Vec<[u8; 3]> = vec![];
//         while i < rgb_image.len() {
//             result.push([
//                 rgb_image[i].into(),
//                 rgb_image[i + 1].into(),
//                 rgb_image[i + 2].into(),
//             ]);
//             i += 3;
//         }
//         Self::new(image.width() as usize, image.height() as usize, result)
//     }
//
//     /// Get the height of the image.
//     pub fn get_height(&self) -> usize {
//         self.height
//     }
//
//     /// Get the width of the image.
//     pub fn get_width(&self) -> usize {
//         self.width
//     }
//
//     /// Get the pixel at the (x,y) position, with [R, G, B] values.
//     pub fn get_rgb_pixel(&self, x: usize, y: usize) -> [u8; 3] {
//         self.image[y * self.width + x]
//     }
//
//     /// Get the pixel at the (x,y) position, with [Y, Ry, By] values.
//     ///
//     /// Refrence is at below:
//     ///
//     /// > Dayton Paper Appendix B: YRyBy (YCrCb) Color Encoding
//     /// >
//     /// > To convert non-linear RGB to [Y, R-Y, B-Y] (scaled to 0-255)
//     /// >
//     /// > $$ Y = 16.0 + (.003906 * ((65.738 * R) + (129.057 * G) + (25.064 * B))) $$
//     /// > $$ RY = 128.0 + (.003906 * ((112.439 * R) + (-94.154 * G) + (-18.285 * B))) $$
//     /// > $$ BY = 128.0 + (.003906 * ((-37.945 * R) + (-74.494 * G) + (112.439 * B))) $$
//     pub fn get_ycrcb_pixel(&self, x: usize, y: usize) -> [u8; 3] {
//         let rgb = self.get_rgb_pixel(x, y);
//
//         let y = 16.0
//             + (0.003906
//                 * ((65.738 * rgb[0] as f32)
//                     + (129.057 * rgb[1] as f32)
//                     + (25.064 * rgb[2] as f32)));
//         let ry = 128.0
//             + (0.003906
//                 * ((112.439 * rgb[0] as f32)
//                     + (-94.154 * rgb[1] as f32)
//                     + (-18.285 * rgb[2] as f32)));
//         let by = 128.0
//             + (0.003906
//                 * ((-37.945 * rgb[0] as f32)
//                     + (-74.494 * rgb[1] as f32)
//                     + (112.439 * rgb[2] as f32)));
//
//         [y as u8, ry as u8, by as u8]
//     }
//
//     /// Resize the image to the new_width and new_height, with the nearest-neighbor
//     /// interpolation algorithm.
//     pub fn resize_image(&self, new_width: usize, new_height: usize) -> Self {
//         let mut resized_img: Vec<[u8; 3]> = vec![[0, 0, 0]; new_width * new_height];
//         for y in 0..new_height {
//             for x in 0..new_width {
//                 // Calculate the corresponding position in the original image
//                 let orig_x = (x as f32 * self.width as f32 / new_width as f32)
//                     .min(self.width as f32 - 1.0)
//                     .round() as usize;
//                 let orig_y = (y as f32 * self.height as f32 / new_height as f32)
//                     .min(self.height as f32 - 1.0)
//                     .round() as usize;
//
//                 // Get the pixel from the original image
//                 let pixel = self.get_rgb_pixel(orig_x, orig_y);
//
//                 // Set the pixel in the resized image
//                 resized_img[y * new_width + x] = pixel;
//             }
//         }
//
//         Self {
//             width: new_width,
//             height: new_height,
//             image: resized_img,
//         }
//     }
// }
//
