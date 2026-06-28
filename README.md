# Reliability History
The missing gui for end users to monitor health of the system over time.  
The goal is to have long term app crashes history without the need to externalize logs, easy to use, efficient, reliable.  
At this time it is compat with Linux journald (most distros).  
Provide a GUI to find out important element in system logs over time.  

With that you have:  
 - classification of journald events by kind  
 - automatic retrieval of containing packages - which provide the package version causing the crash  

# Roadmap
 - write a service that collect important events from journald and store them for longer time
 - extend the scope of the log collector to other OSs (macOS, Windows)

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