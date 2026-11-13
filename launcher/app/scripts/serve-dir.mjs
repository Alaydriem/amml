// Usage: node serve-dir.mjs <dir> <port>
// Static file server bound to 127.0.0.1 only. Logs "<method> <path> <status>" per request.
import { createReadStream, statSync } from "node:fs";
import { createServer } from "node:http";
import { join, normalize, resolve, sep } from "node:path";

class DirServer {
  /**
   * @param {string} root
   * @param {number} port
   */
  constructor(root, port) {
    this.root = resolve(root);
    this.port = port;
  }

  /**
   * @param {string} urlPath
   * @returns {string | null}
   */
  resolveFile(urlPath) {
    let decoded;
    try {
      decoded = decodeURIComponent(urlPath.split("?")[0] ?? "");
    } catch {
      return null;
    }
    const file = normalize(join(this.root, decoded));
    if (file !== this.root && !file.startsWith(this.root + sep)) return null;
    try {
      return statSync(file).isFile() ? file : null;
    } catch {
      return null;
    }
  }

  /**
   * @param {import("node:http").IncomingMessage} req
   * @param {import("node:http").ServerResponse} res
   */
  handle(req, res) {
    const file = req.method === "GET" || req.method === "HEAD" ? this.resolveFile(req.url ?? "/") : null;
    const status = file ? 200 : 404;
    console.log(`${req.method} ${req.url} ${status}`);
    if (!file) {
      res.writeHead(404, { "content-type": "text/plain" });
      res.end("not found\n");
      return;
    }
    res.writeHead(200, {
      "content-type": file.endsWith(".json") ? "application/json" : "application/octet-stream",
      "content-length": statSync(file).size,
    });
    if (req.method === "HEAD") {
      res.end();
      return;
    }
    const stream = createReadStream(file);
    stream.on("error", () => res.destroy());
    stream.pipe(res);
  }

  start() {
    const server = createServer((req, res) => this.handle(req, res));
    server.listen(this.port, "127.0.0.1", () => {
      console.log(`serving ${this.root} on http://127.0.0.1:${this.port}/ (Ctrl+C stops)`);
    });
    process.on("SIGINT", () => server.close(() => process.exit(0)));
    process.on("SIGTERM", () => server.close(() => process.exit(0)));
  }
}

const [dir, portText] = process.argv.slice(2);
const port = Number(portText);
if (!dir || !Number.isInteger(port) || port < 1 || port > 65535) {
  console.error("usage: node serve-dir.mjs <dir> <port>");
  process.exit(2);
}
new DirServer(dir, port).start();
