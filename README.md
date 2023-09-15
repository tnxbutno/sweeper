# Sweeper - File Cleanup Utility

Sweeper is a user-friendly GUI program designed to streamline the process of managing paired JPEG and NEF (Nikon Electronic Format) files. It simplifies identifying and deleting unpaired files, ensuring your photo library stays organized and clutter-free.

## Features

Sweeper quickly identifies files in your chosen directory that are not paired with their corresponding format (JPEG to NEF and vise versa).
For example, if you have cat.jpg, dog.jpg, and dog.nef, this program will delete cat.jpg because there is no cat.nef file.

## How It Works

Sweeper operates based on the following principles:

1. It scans the selected directory for both JPEG and NEF files.

2. Sweeper determines whether a file has a matching counterpart in the other format.

3. Unpaired files are presented to you for confirmation before deletion. This ensures that no important files are removed by accident.

## Getting Started

To use Sweeper, follow these steps:

1. **Run the Scan**: Click the "Scan" button to choose the directory containing your photo files that need cleaning and initiate the file pairing process.

2. **Review and Confirm**: Sweeper will display a list of unpaired files. Review this list carefully, and when you're sure, click "Clean Up!" to remove the unpaired files.

3. **Enjoy an Organized Library**: Your photo library is now cleaner and more organized!

## System Requirements

- Operating System: Linux, MacOS, Windows

## Contributing

Contributions to Sweeper are welcome! If you have ideas for improvements or bug fixes, please open an issue or submit a pull request.

## License

Sweeper is open-source software released under the MIT license. See the LICENSE file for more details.

## Acknowledgments

Sweeper is built with [fltk-rs](https://github.com/fltk-rs/fltk-rs), Rust bindings for the FLTK GUI library.