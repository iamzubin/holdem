import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import axios from 'axios';

// Get the directory name in ES modules
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Read version from package.json
const packageJsonPath = path.join(__dirname, 'package.json');
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
const version = packageJson.version;

// Set the private key for Tauri updater
process.env.TAURI_SIGNING_PRIVATE_KEY = "dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5Y3ZTYUFMUUU1ZWJVdjN3S2p0clI1WUpENzFydkpnYVJiNUk5Sjd0cGFEY0FBQkFBQUFBQUFBQUFBQUlBQUFBQS9tWkRFODIzQWxJS0M3elFUcS9pdmZIT2ZJSUFnWWR3a1U3aDZEWDBicEY2SHZRdzVNUVQ2dHA0SlBjNzhkUit2ZE82OTUzNnhiSE0xZ3pOWjhJQk9kUHlnZUZTN3hSbnYyRTVSK0VKNENYVTFrRFA0WVBFbXB0RVk0QVNLQ0hFQ3UwZlYvYzM1Rk09Cg==";

const token = process.env.GITHUB_TOKEN;
if (!token) {
    console.error("Error: GITHUB_TOKEN environment variable is not set.");
    process.exit(1);
}

const owner = "iamzubin";  // Your GitHub username
const repo = "holdem";      // Repository name

// Helper function to execute shell commands
function execCommand(command) {
    try {
        execSync(command, { stdio: 'inherit' });
    } catch (error) {
        console.error(`Error executing command: ${command}`);
        console.error(error.message);
        process.exit(1);
    }
}

// Helper function to find files
function findFile(dir, pattern) {
    const files = fs.readdirSync(dir);
    const file = files.find(f => f.match(pattern));
    if (!file) {
        throw new Error(`Could not find file matching pattern ${pattern} in ${dir}`);
    }
    return path.join(dir, file);
}

async function main() {
    try {
        // Install dependencies
        console.log("Installing dependencies...");
        execCommand("npm install");

        // Build the application
        console.log("Building application...");
        execCommand("npm run tauri build");

        // Get the installer and signature files
        const nsisDir = path.join("src-tauri", "target", "release", "bundle", "nsis");
        const releaseDir = path.join("src-tauri", "target", "release");

        const installerFile = findFile(nsisDir, new RegExp(`holdem_${version}_x64-setup.exe`));
        const signatureFile = findFile(nsisDir, /\.sig$/);
        const binaryFile = findFile(releaseDir, /\.exe$/);

        // Read the signature
        const signature = fs.readFileSync(signatureFile, 'utf8').trim();

        // Create update.json content
        const updateJson = {
            version,
            notes: `Release notes for Holdem ${version}`,
            pub_date: new Date().toISOString(),
            platforms: {
                "windows-x86_64": {
                    signature,
                    url: `https://github.com/${owner}/${repo}/releases/download/${version}/holdem_${version}_x64-setup.exe`
                }
            }
        };

        // Save update.json
        const updateJsonPath = path.join("..", "holdem_website", "public", "update.json");
        fs.writeFileSync(updateJsonPath, JSON.stringify(updateJson, null, 2), 'utf8');

        // Create the release
        console.log("Creating GitHub release...");
        const releaseData = {
            tag_name: version,
            target_commitish: "main",
            name: `Holdem ${version}`,
            body: `Release notes for Holdem ${version}`,
            draft: true,
            prerelease: false
        };

        const headers = {
            "Accept": "application/vnd.github+json",
            "Authorization": `Bearer ${token}`,
            "X-GitHub-Api-Version": "2022-11-28"
        };

        const release = await axios.post(
            `https://api.github.com/repos/${owner}/${repo}/releases`,
            releaseData,
            { headers }
        );

        console.log("Release created successfully!");

        // Upload update.json
        console.log("Uploading update.json...");
        const updateJsonContent = fs.readFileSync(updateJsonPath);
        await axios.post(
            release.data.upload_url.replace("{?name,label}", "?name=update.json"),
            updateJsonContent,
            {
                headers: {
                    ...headers,
                    "Content-Type": "application/json"
                }
            }
        );
        console.log("Successfully uploaded update.json");

        // Upload the binary
        console.log("Uploading binary...");
        const binaryContent = fs.readFileSync(binaryFile);
        await axios.post(
            release.data.upload_url.replace("{?name,label}", `?name=${path.basename(binaryFile)}`),
            binaryContent,
            {
                headers: {
                    ...headers,
                    "Content-Type": "application/octet-stream"
                }
            }
        );
        console.log("Successfully uploaded binary");

        // Upload the NSIS installer
        console.log("Uploading NSIS installer...");
        const installerContent = fs.readFileSync(installerFile);
        await axios.post(
            release.data.upload_url.replace("{?name,label}", `?name=${path.basename(installerFile)}`),
            installerContent,
            {
                headers: {
                    ...headers,
                    "Content-Type": "application/octet-stream"
                }
            }
        );
        console.log("Successfully uploaded NSIS installer");



        console.log("Release process completed!");
    } catch (error) {
        console.error("Error:", error.message);
        if (error.response) {
            console.error("Response:", error.response.data);
        }
        process.exit(1);
    }
}

main(); 