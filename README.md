# Facharbeit

## DevContainer
Make sure you have [Docker](https://www.docker.com/products/docker-desktop/) and [VS Code](https://code.visualstudio.com/) with the [Remote Extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) installed.

Then simply click on this button:

[![VS Code Container](https://img.shields.io/static/v1?label=VS+Code&message=Container&logo=visualstudiocode&color=007ACC&logoColor=007ACC&labelColor=2C2C32)](https://vscode.dev/redirect?url=vscode://ms-vscode-remote.remote-containers/cloneInVolume?url=https://github.com/sshcrack/enkrypton)


## Virtualisierung


Docker Desktop installieren: [Hier klicken](https://www.docker.com/products/docker-desktop/)
<br>
Zip-Datei entpacken und in Visual Studio Code öffnen.
<br>
Remote Containers installieren: [Hier klicken](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)
<br>
In Visual Studio Code ganz unten links auf folgendes Icon drücken:
![Unten Links icon](https://i.imgur.com/mocKJTw.png)
<br>
Dann auf "Reopen in Container" drücken.
<br>
Konsole öffnen und folgendes eingeben:
Um mit sich selber schreiben zu können:

```bash
yarn tauri dev --features dev
```

Um mit anderen schreiben zu können:
```bash
yarn tauri dev
```

