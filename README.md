# RustyChat

**RustyChat** is a terminal-based asynchronous chat server application written in Rust. It allows users to set up and interact with a chat server directly from their terminal.

---

## 🚀 Features
- Terminal-based chat interface.
- Asynchronous communication.
- Easy setup and usage.

---

## 📥 Installation

### Install via Git
1. Clone the repository:
   ```sh
   git clone https://github.com/yousef-rafat/RustyChat.git
2. Change the directory:
   ```sh
   cd exe
3. Run the executable:
   ```sh
   ./RustyChat.exe -init [PORT]
## 🌐 Connection

Once the RustyChat server runs, you can connect using a telnet client. Follow these steps to join the chat server:

1. Open your terminal.
2. Use the `telnet` command to connect to the server. Replace `8080` with the port number you've configured when starting the RustyChat server.

   Example command:
   ```sh
   telnet 127.0.0.1 8080
   
## ✨ Magic Commands

RustyChat supports a set of **magic commands** that add functionality and enhance the user experience. These commands are invoked directly in the chat interface by typing the command name prefixed with an `&`.

### Supported Magic Commands:
- `&show_users`: Displays the list of currently connected users.
- `&clear_screen`: Clears the chat screen in your terminal.
- `&save_chat`: Saves the current chat log to a file for later reference.
- `&help`: Displays a list of available magic commands and their descriptions.

### Future Commands:
RustyChat is under active development, and more magic commands may be added to extend its capabilities. Stay tuned for updates!
