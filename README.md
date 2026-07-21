# Asvid

Play local videos and YouTube videos directly in your terminal as colored ASCII.

> **Demo**
<p align="center">
  <img src="assets/demo.gif" alt="Asvid Demo" width="100%">
</p>

---

## Requirements

- Rust
- FFmpeg
- `yt-dlp` (required for YouTube videos)

---

## Installation

Clone the repository:

```bash
git clone https://github.com/<your-username>/asvid.git
cd asvid
```

Install the binary:

```bash
cargo install --path .
```

Or run it directly:

```bash
cargo run -- <video-file-or-youtube-url>
```

---

## Usage

Local video:

```bash
asvid movie.mp4
```

YouTube video:

```bash
asvid "https://youtu.be/dQw4w9WgXcQ"
```

---

## Controls

| Key | Action |
|------|--------|
| `Space` | Pause / Resume |
| `←` | Seek backward 5 seconds |
| `→` | Seek forward 5 seconds |
| `q` | Quit |

---

## Features

- Play local video files
- Play YouTube videos using `yt-dlp`
- Render videos as colored ASCII
- Real-time playback synchronized using video timestamps (PTS)
- Pause and resume playback
- Seek forward and backward by 5 seconds
- Automatically scales the output to the current terminal size

---

## Built With

- Rust
- `ffmpeg-next`
- `crossterm`
- `yt-dlp`
