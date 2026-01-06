# Copilot Instructions for auxide-dsp

## Archive Creation Policy
When creating archives, tarballs, or any deliverables:
- Place them in the root directory of the project being archived
- Do not place them in parent directories (like repos/)
- This prevents workflow disruption and confusion

### Standardized Tarball Creation
- **Naming Convention**: Use `{project-name}.tar.gz` (e.g., `auxide.tar.gz`, `auxide-dsp.tar.gz`, `auxide-io.tar.gz`, `auxide-midi.tar.gz`)
- **Creation Command**: From the project root directory, run:
  ```
  cd /path/to/project && rm -f {project-name}.tar.gz && tar -czf {project-name}.tar.gz --exclude=target --exclude=.git --exclude={project-name}.tar.gz --ignore-failed-read .
  ```
- **Cleanup**: Always delete the old tarball before creating a new one to avoid clutter
- **Git Ignore**: Each repository's `.gitignore` includes the standardized tarball names to prevent accidental commits

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