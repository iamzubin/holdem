import { NextResponse } from 'next/server';

export async function GET() {

    const updateData = {
      "version": "0.2.3",
      "notes": "Release notes for Holdem 0.2.3",
      "pub_date": "2025-07-28T19:25:39.317Z",
      "platforms": {
        "windows-x86_64": {
          "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVRWUt1Nm91NG90ZUZXRlpYZlk4WVZrZmZET0hhWlgyYWl5WEszSW1QWVJud204Z3N6Q3cyQzczS0hHaEJyN28vY1dhdWhHcS85dnVVQUdxTVltVE1SQ2w2TkY4WVJFcXcwPQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzUzNzMwNzM5CWZpbGU6aG9sZGVtXzAuMi4zX3g2NC1zZXR1cC5leGUKK3JOY2xDdTVucWQxdk1uemQvQ1dGWjJOeUNNT2hXK1I5cFVKWmk1QVBwVFJqb3FQZlhYZEJIVFdrQ09XQ2ZCSTlZNlpPMk54bTVQTDJCTTdIWCtuQVE9PQo=",
          "url": "https://github.com/iamzubin/holdem/releases/download/0.2.3/holdem_0.2.3_x64-setup.exe"
        }
      }
    };

  return NextResponse.json(updateData);
} 