# Quickstart

Two things, in order. **Part 1** puts a web page on the screen in five minutes, on your own computer, with nothing else needed. **Part 2** puts a real
site on the internet with a proper `https://` padlock, and tests it first so you find out what is wrong before it matters.

> **Honest note.** This page was written before the first release. The commands assume WebServAnt is published (`cargo install webservant`). Until it
> is, build it from the repository instead: `cargo install --path .` run inside a clone of it. Everything below the install line is the same.

You need Rust installed ([rustup.rs](https://rustup.rs)). You do not need to write Rust to use the command-line program, but Part 1 shows the Rust
version too because it is only three lines.

---

## Part 1: a page on your screen (5 minutes)

### The command-line way

1. Install the program:

   ```sh
   cargo install webservant
   ```

2. Make a folder with one page in it:

   ```sh
   mkdir public
   echo "<h1>Hello from my site</h1>" > public/index.html
   ```

3. Serve the folder:

   ```sh
   webservant serve --files ./public
   ```

   **Success looks like this one line**, and the program keeps running until you press Ctrl-C:

   ```
   serving http://127.0.0.1:8080/  (Ctrl-C to stop)
   ```

4. Open <http://127.0.0.1:8080/> in a browser. You see "Hello from my site". Press Ctrl-C in the terminal to stop.

It listens on this computer only (`127.0.0.1`), so nobody else can reach it. That is on purpose.

### The Rust way (three lines)

```sh
cargo new mysite
cd mysite
cargo add webservant
cargo add tokio --features macros,rt-multi-thread
mkdir public
echo "<h1>Hello from my site</h1>" > public/index.html
```

Put this in `src/main.rs`:

```rust
#[tokio::main]
async fn main() -> webservant::Result<()> {
    webservant::Site::new("example.org").files("./public").serve().await
}
```

Then `cargo run` and open <http://127.0.0.1:8080/>. Same result.

**If something goes wrong in Part 1**

| You see | It means | Do this |
|---|---|---|
| `webservant: cannot serve files from ...` | the folder in `--files` does not exist | check you ran step 2 in the folder you are in |
| `the server could not start or stopped: ... address already in use` | something else uses port 8080 | pick another: `--bind 127.0.0.1:8081`, then open that port |
| `unknown option ...` | a typo in a flag | WebServAnt refuses typos on purpose; fix the spelling shown |
| the browser says "refused to connect" | the program is not running | look at the terminal: is it still printing nothing and waiting? If it exited, read its last line |

---

## Part 2: a real site with HTTPS

What happens: WebServAnt asks a certificate authority (Let's Encrypt) for a certificate for your name, proves the name is yours by answering a
question the authority sends to port 80 on your machine, and then serves your folder over HTTPS and renews the certificate by itself. For that to work,
**the internet must be able to reach your machine on port 80 and port 443**. Most problems in this part are about that, which is why there is a test
first (step 6).

### 1. What you need

- A **domain name** you control, for example `example.org`.
- A **computer on the internet with its own public address** (a small rented server works; a laptop at home usually does not unless you can open ports
  on your router). You can log into it and install software.
- About 20 minutes.

### 2. Point the name at the computer

In the place where you manage your domain, add an **A record**: name `example.org` (or `@`), value = the computer's **public IPv4 address**. Do the
same for `www` if you want `www.example.org` too.

Check it from any computer:

```sh
nslookup example.org
```

The address shown must be your computer's public address. DNS changes can take minutes to hours to spread.

**If your domain has an AAAA record** (an IPv6 address) that does not lead to this computer, remove it. The certificate authority may try that address
and fail.

### 3. Open ports 80 and 443

Both must be reachable from the internet.

- On the computer's own firewall. For example on Ubuntu: `sudo ufw allow 80/tcp` and `sudo ufw allow 443/tcp`.
- In your hosting company's control panel, if it has a "firewall" or "security group" (many do and they block everything by default): allow inbound
  TCP 80 and 443.
- At home: forward ports 80 and 443 on your router to the computer.

Only one program can use port 80 at a time. If a web server (nginx, Apache, IIS) already runs on this computer, stop it for now.

### 4. Install and make a page

```sh
cargo install webservant
mkdir public
echo "<h1>My real site</h1>" > public/index.html
```

On Linux, ports below 1024 need permission. Give the program that permission once (do not run it as root):

```sh
sudo setcap 'cap_net_bind_service=+ep' "$(which webservant)"
```

On Windows, allow it through the firewall when Windows asks.

### 5. Choose what to call yourself

You will use two things below: your **name** (`example.org`) and an **address the certificate authority can write to** (`you@example.org`). Replace both
in every command.

### 6. Test first (this is the important step)

```sh
webservant check --name example.org --acme you@example.org --files ./public
```

This does **one real test issuance against Let's Encrypt's practice service (staging)**. It uses a throwaway store, keeps nothing, changes nothing on
your computer when it finishes, and cannot use up your real limits. It takes from a few seconds to a minute.

**Success looks like this one line and the command ends with no error:**

```
OK: https://acme-staging-v02.api.letsencrypt.org/directory issued a certificate for example.org. Nothing was kept.
```

If you see that, your name, your ports and your firewall are all right. Go to step 7.

**If it fails**, the command prints one line starting with `NOT REACHABLE` or `webservant:` and stops. Find yours:

| You see (the start of the line) | It means | Do this |
|---|---|---|
| `NOT REACHABLE (...): ... could not validate the name ... port 80` | the authority could not reach your computer on port 80, or reached something else | repeat steps 2 and 3: does `nslookup` show this computer's public address? is port 80 open in **every** firewall (computer, hosting panel, router)? is another web server using port 80? |
| `webservant: the server could not start or stopped: ... address already in use` | another program already uses port 80 or 443 | find and stop it (`sudo ss -ltnp` on Linux lists who listens where), then run the test again |
| `webservant: the server could not start or stopped: ... Permission denied` | no permission for ports below 1024 | run the `setcap` command in step 4 (Linux) and try again |
| `NOT REACHABLE ...: the certificate authority refused or failed at ...` and the rest mentions a limit (`rate`, `too many`) | the practice service is limiting you (the exact wording is the authority's, and this page has not seen it) | wait an hour and try again; do not run the test in a loop |
| `unknown option ...` or `... needs a value` | a typo or a missing value in the command | read the line: it names the flag |
| anything else | a case this page does not cover | copy the **whole** line into a bug report at the project's issue tracker |

Run the test as many times as you need while you fix things. Each run is independent.

### 7. Go live

Now the real thing. This is the same command as `serve` with the same flags, **without** `check` and without `--staging`:

```sh
webservant serve --files ./public --name example.org --acme you@example.org
```

The first start takes up to a minute while it gets the certificate. **Success looks like this line**, and the program keeps running:

```
serving https://0.0.0.0:443/  (Ctrl-C to stop)
```

Open `https://example.org` in a browser. You should see "My real site" and a padlock. Visiting `http://example.org` sends you to the `https://` address.

If you would rather start with the practice certificates (browsers will warn that they are not trusted, which is expected), add `--staging` to the
command. Leave it off for the real site.

### 8. Keep it running

- **Renewal is automatic.** The certificate is renewed by itself when a third of its life is left, with no restart. If a renewal fails the current
  certificate keeps being served and the problem is retried.
- **Where things are kept.** The account and certificates (they contain private keys) are in a folder only your user can read: on Linux
  `~/.local/share/webservant`, on Windows `%LOCALAPPDATA%\webservant`. Choose another place with `--acme-store /some/folder`.
- **Stopping.** Ctrl-C, or `SIGTERM` (what `systemctl stop` sends), stops it cleanly.
- **Starting on boot (Linux, systemd).** A starting point, **not yet run on a real server by the author**, so read it before using it. Save as
  `/etc/systemd/system/webservant.service`:

  ```ini
  [Unit]
  Description=WebServAnt
  After=network-online.target
  Wants=network-online.target

  [Service]
  ExecStart=/usr/local/bin/webservant serve --files /srv/site --name example.org --acme you@example.org --acme-store /var/lib/webservant
  DynamicUser=yes
  StateDirectory=webservant
  AmbientCapabilities=CAP_NET_BIND_SERVICE
  Restart=on-failure

  [Install]
  WantedBy=multi-user.target
  ```

  then `sudo systemctl enable --now webservant`. (`/srv/site` must be readable by the service, and the program installed at that path.)

### 9. Put your own app behind it

If you have an app listening on this computer, say on port 3000, send the paths it should answer to it:

```sh
webservant serve --files ./public --name example.org --acme you@example.org --proxy /api=http://127.0.0.1:3000
```

Or give it everything the folder does not have (files that exist are served first):

```sh
webservant serve --files ./public --name example.org --acme you@example.org --proxy /=http://127.0.0.1:3000
```

---

## What this does not do (yet)

Wildcard certificates, getting certificates on demand for names you did not list, forwarding WebSocket connections, directory listings, logging in
visitors, compression. The list and the reasons are in the [README](../README.md) and [ROADMAP](../ROADMAP.md). Nothing has been run against Let's
Encrypt's practice service from this project's own machines yet: step 6 is exactly that run, which is why it comes first.
