# Quickstart

Two things, in order. **Part 1** puts a web page on your screen, on your own computer, with nothing else needed (about five minutes, most of it the
computer compiling). **Part 2** puts a real site on the internet with a proper `https://` padlock, and tests it first so you find out what is wrong
before it matters.

You need Rust installed ([rustup.rs](https://rustup.rs)). You do not need to write Rust to use the command-line program, but Part 1 shows the Rust
version too because it is three lines.

The first install takes a couple of minutes while Rust compiles it. You will see a lot of "Compiling" lines; that is normal.

---

## Part 1: a page on your screen

### The command-line way

1. Install the program (a couple of minutes of compiling):

   ```sh
   cargo install webservant
   ```

   A line starting with `Installing` near the end of cargo's output says where the program went (normally the `bin` folder inside your cargo home, `~/.cargo/bin`, which `rustup` puts on your
   PATH). If your terminal then says `webservant` is not found, close it and open a new one.

2. Make a folder with one page in it:

   ```sh
   mkdir public
   echo "<h1>Hello from my site</h1>" > public/index.html
   ```

   (Git Bash and PowerShell 7 run these two lines as written. In older Windows PowerShell or cmd, `echo` can write a file your browser misreads, so make the
folder `public` and the file `index.html` in it with any text editor instead; the file only needs the one line.)

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

Now **replace everything** in `src/main.rs` with this:

```rust
#[tokio::main]
async fn main() -> webservant::Result<()> {
    webservant::Site::new("example.org").files("./public").serve().await
}
```

Run `cargo run` (the first build takes a couple of minutes; `cargo add` printed a long list of optional features earlier, which is normal). **This
version prints nothing of its own when it works**: after cargo's `Running ...` line the program is simply running. Open
<http://127.0.0.1:8080/> and you see the same page; press Ctrl-C to stop. (The name `example.org` only matters later, in Part 2; in this local mode it
does nothing.)

### If something goes wrong in Part 1

| You see | It means | Do this |
|---|---|---|
| `webservant: cannot serve files from ...` | the folder in `--files` does not exist | make the folder first, and run the command from the folder that contains it |
| `webservant: there is nothing to serve ...` | you left out `--files` | add `--files ./public` |
| `the server could not start or stopped: ... address already in use` (on Windows: `Only one usage of each socket address ... (os error 10048)`) | something else already uses port 8080 | pick another port: `--bind 127.0.0.1:8081`, then open that port instead |
| `bind ... is not an address` | `--bind` needs the host, a colon and the port | write it like `127.0.0.1:8081` (not just `8081`), with a port from 0 to 65535 |
| the page loads at `localhost:8082` but not at `127.0.0.1:8082` | `--bind localhost:8082` may listen on the IPv6 address only (the `serving` line shows `[::1]`) | use `--bind 127.0.0.1:8082`, or open the address the `serving` line shows |
| `unknown option ...` | a typo in a flag | the message names the flag and usually says what you meant; `webservant --help` lists every option |
| the browser says "refused to connect" or "can't reach this page" | nothing is listening there | is the program still running in its terminal? (The command-line way prints the `serving` line when it is up; the Rust way prints nothing.) Is the port in your browser the one it uses? |

---

## Part 2: a real site with HTTPS

This part is written for a **Linux server**. It has been run once end to end through step 6 on Debian 13; the same commands should work elsewhere, but it has not been tried on other systems.

What happens: WebServAnt asks a certificate authority (Let's Encrypt) for a certificate for your name, proves the name is yours by answering a
question the authority sends to port 80 on your machine, and then serves your folder over HTTPS and renews the certificate by itself. For that to
work, **the internet must be able to reach your machine on port 80 and port 443**. Most problems in this part are about that, which is why there are
two tests before the real thing (step 5, optional but quick, and step 6).

### 1. What you need

- A **domain name** you control, for example `example.org`.
- A **computer on the internet with its own public address** (a small rented server works; a laptop at home usually does not unless you can open ports
  on your router). You can log into it and install software.
- An **email address you can read**, for the certificate authority to write to about expiry and problems.
- **Rust and git on that computer.** On a fresh Ubuntu server: `sudo apt install build-essential git curl`, then install Rust with
  `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` and open a new terminal (the installer may end with some `CFLAGS` lines; they are harmless) (the compiler package is what Rust links with).
- About 20 minutes of work, plus waiting for DNS to spread (step 2).

### 2. Point the name at the computer

In the place where you manage your domain, add an **A record**: name `example.org` (or `@`), value = the computer's **public IPv4 address** (your hosting
company's control panel shows it; on the computer itself `curl https://ifconfig.me` prints it). Do the same for `www` if you want `www.example.org` too, **and then add `--also-name www.example.org` to the commands in steps 6 and 7**: one
certificate covers exactly the names you list, and a name you did not list gets a certificate error.

Check it from any computer (change `example.org` to your name; if `nslookup` is missing on a minimal Ubuntu server, `sudo apt install dnsutils`):

```sh
nslookup example.org
nslookup -type=AAAA example.org
```

The first must show your computer's public address. DNS changes can take minutes to hours to spread. **The second should show no address** (it may say
"No answer" or that the name does not exist for that type; that is what you want). If your
domain has an AAAA record (an IPv6 address) that does not lead to this computer, remove it: the certificate authority may try that address and fail.

### 3. Open ports 80 and 443

Both must be reachable from the internet.

- On the computer's own firewall, if it has one (some servers, such as a stock Debian 13, have no `ufw` and accept everything: then there is nothing to do here). For example on Ubuntu: `sudo ufw allow 80/tcp` and `sudo ufw allow 443/tcp`. **If you turn `ufw` on, allow SSH first
  (`sudo ufw allow 22/tcp`) or you lock yourself out of the server.**
- In your hosting company's control panel, if it has a "firewall" or "security group" (many do and they block everything by default): allow inbound
  TCP 80 and 443.
- At home: forward ports 80 and 443 on your router to the computer.

Only one program can use port 80 at a time. If a web server (nginx, Apache) already runs on this computer, stop it for now.

### 4. Install, give it permission for ports 80 and 443, make a page

```sh
cargo install webservant
sudo cp "$(which webservant)" /usr/local/bin/webservant
sudo setcap 'cap_net_bind_service=+ep' /usr/local/bin/webservant
cargo uninstall webservant
mkdir public
echo "<h1>My real site</h1>" > public/index.html
```

The `cp` puts the program somewhere `sudo` can see it; the `setcap` line lets it use ports below 1024 without the program itself running as root (that is the point; only the two `sudo` lines run as root);
`cargo uninstall` removes the first copy so that `webservant` from now on is the one with the permission. (After you upgrade WebServAnt later, repeat
the `cargo install`, `cp` and `setcap` lines: a new copy has lost the permission.)

### 5. First quick test: can the internet reach port 80?

This is optional but takes one minute and catches the most common mistake before the real test. Start a plain server on port 80:

```sh
webservant serve --files ./public --bind 0.0.0.0:80
```

You should see a line like the one in Part 1, `serving http://0.0.0.0:80/  (Ctrl-C to stop)`. Now, **from a different network**, for example your phone on mobile data (not your home wifi), open `http://example.org/` (your name). You should see
"My real site". If you do, your DNS and port 80 are right; press Ctrl-C and go on. If you do not, fix steps 2 and 3 first; the real test (step 6) would
fail for the same reason. **Either way, press Ctrl-C to stop this server before step 6**: the test needs port 80 free.

### 6. The real test (this is the important step)

```sh
webservant check --name example.org --acme you@example.org --files ./public
```

This does **one real test issuance against Let's Encrypt's practice service (staging)**: it always uses staging, never the real service. It uses a
throwaway store, keeps nothing, changes nothing on your computer when it finishes, and cannot use up your real limits. It takes from a few seconds to
a minute. For that time it starts the server on ports 80 and 443, which is why they must be free.

**Success looks like this one line**, and the command ends with no error:

```
OK: https://acme-staging-v02.api.letsencrypt.org/directory issued a certificate for example.org. Nothing was kept.
```

If you see that, your name, your ports and your firewall are all right. Go to step 7.

**If it fails**, it prints a message and stops. The messages start in a few different ways; find yours:

| You see (the start of the message) | It means | Do this |
|---|---|---|
| `NOT REACHABLE (...): ... could not validate the name ... port 80` | the authority could not reach your computer on port 80, or reached something else | repeat steps 2, 3 and 5: does `nslookup` show this computer's public address? is port 80 open in **every** firewall (computer, hosting panel, router)? is another web server using port 80? |
| `webservant: the server could not start or stopped: ... address already in use` | another program already uses port 80 or 443 | find and stop it (`sudo ss -ltnp` lists who listens where), then run the test again |
| `webservant: the server could not start or stopped: ... Permission denied` | no permission for ports below 1024 | run the `setcap` line in step 4 on the copy you are running, and try again |
| `NOT REACHABLE ...: the certificate authority refused or failed at ...` and the rest mentions a limit (`rate`, `too many`) | the practice service is limiting you (the exact wording is the authority's, and this page has not seen it) | wait an hour and try again; do not run the test in a loop |
| `unknown option ...` or `... needs a value` | a typo or a missing value in the command | the message names the flag and may say what you meant |
| anything else | a case this page does not cover | copy the **whole** message into a bug report at <https://github.com/ThinkersJournal/webservant/issues> |

If you set up `www` too (step 2), test both names in one go:

```sh
webservant check --name example.org --also-name www.example.org --acme you@example.org --files ./public
```

Run the test as many times as you need while you fix things. Each run is independent.

### 7. Go live

Now the real thing: the same flags, but `serve` instead of `check`:

```sh
webservant serve --files ./public --name example.org --acme you@example.org
```

This one uses the real Let's Encrypt service, which is why you tested first. The first start takes up to a minute while it gets the certificate. It listens on port 80 too, to send `http://` visitors to `https://`.
**Success looks like this line**, and the program keeps running (`0.0.0.0` just means "every address of this computer"; you open your own name, below):

```
serving https://0.0.0.0:443/  (Ctrl-C to stop)
```

(With `www`: add `--also-name www.example.org` after `--name example.org`, exactly as in the test.)

Open `https://example.org` in a browser. You should see "My real site" and a padlock. Visiting `http://example.org` sends you to the `https://`
address.

### 8. Keep it running

- **Renewal is automatic.** The certificate is renewed by itself when a third of its life is left, with no restart. If a renewal fails the current
  certificate keeps being served and the problem is retried.
- **Where things are kept.** The account and certificates (they contain private keys) are in a folder only your user can read: on Linux
  `~/.local/share/webservant`. Choose another place with `--acme-store /some/folder`.
- **Stopping.** Ctrl-C, or `SIGTERM` (what `systemctl stop` sends), stops it cleanly.
- **Starting on boot (systemd).** A starting point, **not yet run on a real server by the author**, so read it before using it. Save as
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

  **Stop the server from step 7 first (Ctrl-C):** the service needs ports 80 and 443. Then copy your site into place and start it:

  ```sh
  sudo mkdir -p /srv/site
  sudo cp -r public/. /srv/site/
  sudo systemctl daemon-reload
  sudo systemctl enable --now webservant
  ```

  Because of `AmbientCapabilities` the `setcap` line is not needed for the service. The service keeps its **own** store (`/var/lib/webservant`), so it
  orders its own certificate; do not switch back and forth between running it by hand and as a service many times, or you will use up the real
  service's limits.

### 9. Put your own app behind it

If you have an app listening on this computer, say on port 3000, send the paths it should answer to it. (If you run WebServAnt as a service, step 8,
put these flags on the `ExecStart=` line instead, then `sudo systemctl daemon-reload && sudo systemctl restart webservant`.)

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
visitors, compression. The list and the reasons are in the [README](https://github.com/ThinkersJournal/webservant/blob/main/README.md) and
[ROADMAP](https://github.com/ThinkersJournal/webservant/blob/main/ROADMAP.md). Nothing has been run against Let's Encrypt's practice service from this
project's own machines yet: step 6 is exactly that run, which is why it comes first.
