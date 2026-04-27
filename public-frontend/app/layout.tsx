import type { Metadata } from "next";

import "./globals.css";

export const metadata: Metadata = {
  title: "Verdant Agent Studio",
  description: "AI-native public homepage and member console for PopTail-admin.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="zh-CN">
      <body className="grain">{children}</body>
    </html>
  );
}
