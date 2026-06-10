# Reliability History - RelHist
The missing gui for end users to monitor health of the system over time.  
The goal is to have long term app crashes history without the need to externalize logs, easy to use, efficient, reliable, compat any distro that includes journald.

## Developpment
### Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

### Build release

```bash
npm run tauri build
```

### Run dev 
```bash
npm run tauri dev
```