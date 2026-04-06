import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "My App",
  description: "Built with dex + Next.js on Databricks",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
