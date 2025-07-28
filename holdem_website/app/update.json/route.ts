import { NextResponse } from 'next/server';

export async function GET() {

    const updateData = {
      "version": "0.2.3",
      "notes": "Release notes for Holdem 0.2.3",
      "pub_date": "2025-07-28T18:12:36.669Z",
      "platforms": {
        "windows-x86_64": {
          "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVRWUt1Nm91NG90ZUtIL1JGbTlGTDIxZ0wwQzluaDBocHJKclpRK3BrK0RtanBkcXhyTDB5cThSK1AxMUt0bUY0QmJoV0ZOQkZSNU9HUGRnV0NIQTRJOGYwaGxXSDVHMVFvPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzUzNzI2MTU3CWZpbGU6aG9sZGVtXzAuMi4yX3g2NC1zZXR1cC5leGUKaU1rK3VoUnNYUHp4cnNOZXAzY29HVGgrRmhqcGE2SEpyS2djRk5hUXR5a2s3T0twRi9Mdmg5MU1aclV4eGZlMERIQkNFMlV1Zmg2cHhwZXpzWW4wRHc9PQo=",
          "url": "https://github.com/iamzubin/holdem/releases/download/0.2.3/holdem_0.2.3_x64-setup.exe"
        }
      }
    };

  return NextResponse.json(updateData);
} 