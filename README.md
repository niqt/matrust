# Matrust
Matrix Rust Client

The goal of this project is to create a Matrix client in Rust using the matrix-sdk and the Slint library.
- https://github.com/matrix-org/matrix-rust-sdk
- https://slint.dev

## Getting Started

To get started, you need to have Rust and Cargo installed on your system. You can download them from the official Rust website: https://www.rust-lang.org/tools/install

Also you have to install the development library of SQLite. On Debian-like systems this can be done with:
```
sudo apt install libsqlite3-dev
```

## Current Features

- Login
- Receive message

## Final Features

- User registration and login
- Room creation and joining
- Sending and receiving messages
- File sharing

## Install Matrix server

The easiest way is to use a Matrix server via Docker. For simplicity, this setup uses an SQLite database instead of the PostgreSQL database typically used in production environments.

Below is a summary of the steps, based on the official instructions found here: https://hub.docker.com/r/matrixdotorg/synapse

### Install docker

The installation method depends on your operating system. Follow the instructions on the official Docker website for your OS.

### Download the docker image

```
docker pull matrixdotorg/synapse
```

### Generate initial configuration

```
docker run -it --rm --mount type=volume,src=synapse-data,dst=/data -e SYNAPSE_SERVER_NAME=my.matrix.host -e SYNAPSE_REPORT_STATS=yes matrixdotorg/synapse:latest generate
```

### Run the docker image

```
docker run -d --name synapse -p 8008:8008 -v /path/to/synapse:/data matrixdotorg/synapse
```

### Create users

```
docker exec -it synapse matrix-admin create-user --admin --password=your_password your_username
```

Repeat this command to create additional users as needed.


### Conclusion

Now everything is up and running (the server is listening on http://your_ip:8008). If you shut down Docker, remember to restart both Docker and the container.
