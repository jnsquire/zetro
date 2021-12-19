import { IZetroClient } from "./generated/code_generated";

export const client: IZetroClient = {
  async makeRequest(body: any) {
    try {
      const r = await fetch("/api", {
        method: "POST",
        body: JSON.stringify(body),
        headers: {
          "Content-Type": "application/json",
        },
      });

      if (r.status != 200) {
        // Status is always 200 if the server is working correctly.
        // Consider this an unexpected error
        alert(`Unexpected error:\n${await r.text()}`);
      } else {
        return await r.json();
      }
    } catch (e) {
      alert(`Could not fetch:\n${e}`);
    }
  },
};
