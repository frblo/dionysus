#!/usr/bin/env python3
"""Registers claims for a subject on the local oidc-provider-mock (see
compose.yml), so logging in as that subject on the mock's "Custom user" form
picks up the given email/email_verified/name.
"""

import argparse
import json
import sys
import urllib.error
import urllib.request

DEFAULT_MOCK_URL = "http://localhost:9400"


def seed_user(
    mock_url: str, subject: str, email: str, verified: bool, name: str
) -> None:
    body = json.dumps(
        {"email": email, "email_verified": verified, "name": name}
    ).encode()
    req = urllib.request.Request(
        f"{mock_url}/users/{subject}",
        data=body,
        method="PUT",
        headers={"Content-Type": "application/json"},
    )
    try:
        urllib.request.urlopen(req)
    except urllib.error.URLError as e:
        sys.exit(f"Failed to seed '{subject}' on {mock_url}: {e}")


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""\
examples:
  # A plain user with a verified email
  %(prog)s alice@example.com

  # A second identity colliding with alice's verified email (different
  # subject, same email)
  %(prog)s alice-work-account --email alice@example.com

  # A user whose email is present but not verified
  %(prog)s carol@example.com --unverified
""",
    )
    parser.add_argument("subject", help="the OIDC subject to register")
    parser.add_argument("-e", "--email", help="email claim (default: <subject>)")
    parser.add_argument(
        "-u",
        "--unverified",
        action="store_true",
        help="mark the email as NOT verified (default: verified)",
    )
    parser.add_argument("-n", "--name", help="display name claim (default: <subject>)")
    parser.add_argument(
        "--mock-url",
        default=DEFAULT_MOCK_URL,
        help=f"base URL of the oidc-provider-mock (default: {DEFAULT_MOCK_URL})",
    )
    args = parser.parse_args()

    email = args.email or args.subject
    name = args.name or args.subject
    verified = not args.unverified

    seed_user(args.mock_url, args.subject, email, verified, name)

    print(
        f"Seeded subject '{args.subject}' (email={email}, email_verified={verified}, name={name})"
    )
    print(
        f"Log in at http://localhost:8000/login, choose 'dev', and enter '{args.subject}' as the subject."
    )


if __name__ == "__main__":
    main()
