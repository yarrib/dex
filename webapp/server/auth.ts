import { Router } from "express";

export const authRouter = Router();

const GITHUB_CLIENT_ID = process.env.GITHUB_CLIENT_ID || "";
const GITHUB_CLIENT_SECRET = process.env.GITHUB_CLIENT_SECRET || "";

/**
 * POST /api/auth/github/device
 * Start the GitHub Device Flow — returns a user code and verification URL.
 * The user enters the code at github.com/login/device to authorize.
 * This works inline in a chat context without popups or redirects.
 */
authRouter.post("/github/device", async (_req, res) => {
  if (!GITHUB_CLIENT_ID) {
    res.status(500).json({ error: "GitHub OAuth not configured. Set GITHUB_CLIENT_ID." });
    return;
  }

  try {
    const response = await fetch("https://github.com/login/device/code", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
      },
      body: JSON.stringify({
        client_id: GITHUB_CLIENT_ID,
        scope: "repo user:email",
      }),
    });

    const data = (await response.json()) as {
      device_code: string;
      user_code: string;
      verification_uri: string;
      expires_in: number;
      interval: number;
    };

    res.json({
      deviceCode: data.device_code,
      userCode: data.user_code,
      verificationUri: data.verification_uri,
      expiresIn: data.expires_in,
      interval: data.interval,
    });
  } catch (err) {
    res.status(500).json({ error: `Device flow initiation failed: ${err}` });
  }
});

/**
 * POST /api/auth/github/device/poll
 * Poll for the device flow token. Called periodically by the frontend.
 */
authRouter.post("/github/device/poll", async (req, res) => {
  const { deviceCode } = req.body as { deviceCode?: string };
  if (!deviceCode) {
    res.status(400).json({ error: "Missing deviceCode" });
    return;
  }

  try {
    const response = await fetch("https://github.com/login/oauth/access_token", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
      },
      body: JSON.stringify({
        client_id: GITHUB_CLIENT_ID,
        device_code: deviceCode,
        grant_type: "urn:ietf:params:oauth:grant-type:device_code",
      }),
    });

    const data = (await response.json()) as {
      access_token?: string;
      error?: string;
      error_description?: string;
    };

    if (data.access_token) {
      // Token granted — fetch user info
      const userRes = await fetch("https://api.github.com/user", {
        headers: { Authorization: `Bearer ${data.access_token}` },
      });
      const userData = (await userRes.json()) as {
        login: string;
        name: string | null;
        avatar_url: string;
      };

      res.json({
        status: "complete",
        token: data.access_token,
        user: {
          login: userData.login,
          name: userData.name,
          avatarUrl: userData.avatar_url,
        },
      });
    } else if (data.error === "authorization_pending") {
      res.json({ status: "pending" });
    } else if (data.error === "slow_down") {
      res.json({ status: "slow_down" });
    } else if (data.error === "expired_token") {
      res.json({ status: "expired" });
    } else {
      res.json({ status: "error", error: data.error_description || data.error });
    }
  } catch (err) {
    res.status(500).json({ error: `Poll failed: ${err}` });
  }
});

/**
 * GET /api/auth/github/url (legacy — kept for fallback)
 * Returns the standard OAuth authorization URL for redirect-based flow.
 */
authRouter.get("/github/url", (_req, res) => {
  if (!GITHUB_CLIENT_ID) {
    res.status(500).json({ error: "GitHub OAuth not configured. Set GITHUB_CLIENT_ID." });
    return;
  }
  const params = new URLSearchParams({
    client_id: GITHUB_CLIENT_ID,
    scope: "repo user:email",
    redirect_uri: process.env.GITHUB_REDIRECT_URI || "",
  });
  res.json({ url: `https://github.com/login/oauth/authorize?${params}` });
});

/**
 * POST /api/auth/github/callback (legacy — kept for redirect-based flow)
 * Exchange an OAuth code for an access token.
 */
authRouter.post("/github/callback", async (req, res) => {
  const { code } = req.body as { code?: string };
  if (!code) {
    res.status(400).json({ error: "Missing code parameter" });
    return;
  }

  try {
    const tokenRes = await fetch("https://github.com/login/oauth/access_token", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
      },
      body: JSON.stringify({
        client_id: GITHUB_CLIENT_ID,
        client_secret: GITHUB_CLIENT_SECRET,
        code,
      }),
    });
    const tokenData = (await tokenRes.json()) as { access_token?: string; error?: string };

    if (!tokenData.access_token) {
      res.status(400).json({ error: tokenData.error || "Failed to get access token" });
      return;
    }

    const userRes = await fetch("https://api.github.com/user", {
      headers: { Authorization: `Bearer ${tokenData.access_token}` },
    });
    const userData = (await userRes.json()) as {
      login: string;
      name: string | null;
      avatar_url: string;
    };

    res.json({
      token: tokenData.access_token,
      user: {
        login: userData.login,
        name: userData.name,
        avatarUrl: userData.avatar_url,
      },
    });
  } catch (err) {
    res.status(500).json({ error: `OAuth exchange failed: ${err}` });
  }
});
