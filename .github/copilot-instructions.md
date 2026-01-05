# Copilot Instructions for auxide-dsp

## Archive Creation Policy
When creating archives, tarballs, or any deliverables:
- Place them in the root directory of the project being archived
- Do not place them in parent directories (like repos/)
- This prevents workflow disruption and confusion

## Terminal Context Awareness
Terminal prompt format: `micha@MikesPC MINGW64 ~/repos $`
- Shows username: micha
- Host: MikesPC
- Shell: MINGW64
- Current directory: ~/repos (which is /c/Users/micha/repos)
- Always check CWD before operations

## Project Structure
This workspace contains:
- auxide/ (kernel)
- auxide-dsp/ (DSP nodes)
- auxide-io/ (audio I/O)
- auxide-midi/ (MIDI - upcoming)

All project-specific outputs should remain within their respective directories.