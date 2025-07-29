import { NextResponse } from 'next/server';

export async function GET() {

    const updateData = {
      "version": "0.2.3",
      "notes": "Release notes for Holdem 0.2.3",
      "pub_date": "2025-07-29T15:56:17.352Z",
      "platforms": {
        "windows-x86_64": {
          "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVRWUt1Nm91NG90ZU1Mbnd5SnlxQjhSRWEzT1BaNTZpaGViNDA0UXVGc3U4VDh5MTRIbS9nU1NpNEwwbGtIakRDTndKdVEwcXd2amVMUktxRVhjcnJhRVdSblA5VkZGV1FNPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzUzODA0NTc3CWZpbGU6aG9sZGVtXzAuMi4zX3g2NC1zZXR1cC5leGUKZGN3akJrTWhKYStSTEE3eG4wNHNQb3dLa29BZDBuU0FIanVsaSszQ3pja2JZNFFBTFFRb1FoYXF4RlZ6V09JYk1URFlSWGNUT3R0cC9XRkpuL1V1Q3c9PQo=",
          "url": "https://github.com/iamzubin/holdem/releases/download/0.2.3/holdem_0.2.3_x64-setup.exe"
        }
      }
    }

  return NextResponse.json(updateData);
} 