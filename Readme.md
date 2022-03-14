# Kiraz

A tool to edit and upload screenshots.

Supported image formats:
- PNG
- JPEG
- GIF
- BMP
- ICO
- WebP
- PNM
- TGA

Supported targets:
- [ ] Save to a file
- [ ] Save to clipboard
- [ ] Upload to imgur
- [ ] Upload to B2
- [ ] Upload to S3
- [ ] Upload to Google Drive
- [ ] Run custom scripts


## Debugging

Run kiraz with the environment variable `KIRAZ_LOG_LEVEL=debug` to see the debug
output.

```bash
env KIRAZ_LOG_LEVEL=debug kiraz image.png
```
