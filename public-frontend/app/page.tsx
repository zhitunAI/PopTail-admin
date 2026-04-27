"use client";

import {
  ChevronDown,
  CopyPlus,
  Folder,
  Grid2X2,
  Heart,
  Home,
  Image as ImageIcon,
  Menu,
  Plus,
  Search,
  SlidersHorizontal,
  Sparkles,
  Upload,
  WandSparkles,
} from "lucide-react";
import { useEffect, useState } from "react";

import { cn } from "@/lib/utils";

type PublicSettings = {
  logoUrl: string;
  siteName: string;
  siteSlogan: string;
  siteDescription: string;
};

const sidebarItems = [
  { icon: Home, label: "灵感", active: true },
  { icon: Sparkles, label: "生成" },
  { icon: Folder, label: "资产" },
  { icon: Grid2X2, label: "画布" },
];

const tools = [
  {
    badge: "Octo",
    badgeClass: "bg-[linear-gradient(135deg,#f7f9ff,#d9e5ff)] text-slate-900",
    label: "Octo",
    meta: "人与AI同屏，共创下一个故事",
    tag: "Beta",
  },
  {
    badge: "",
    badgeClass: "bg-[radial-gradient(circle_at_35%_25%,#bde9ff,#6f7cff_48%,#1b2143)]",
    label: "无限画布",
    meta: "自由创作",
  },
  {
    badge: "",
    badgeClass: "bg-[radial-gradient(circle_at_35%_30%,#d1fff5,#00c6dc_42%,#1a356d)]",
    label: "Agent 模式",
    meta: "S2.0 视频创作",
  },
  {
    badge: "4.1",
    badgeClass: "bg-[linear-gradient(135deg,#14c8ee,#0b5978)] text-cyan-50",
    label: "图片生成",
    meta: "智能美学提升",
  },
  {
    badge: "2.0",
    badgeClass: "bg-[linear-gradient(135deg,#118cff,#0b328d)] text-cyan-50",
    label: "视频生成",
    meta: "Seedance 2.0",
  },
];

const tabs = ["灵感", "短片", "活动"];

const gallery = [
  {
    author: "苹果🍎",
    likes: 192,
    ratio: "aspect-video",
    footer: "大学生广告艺术节学院奖 即梦AI青年创意大赛（视频类）",
    kicker: "web_activity_participants",
    style:
      "bg-[linear-gradient(180deg,rgba(255,255,255,0.04),rgba(0,0,0,0.48)),radial-gradient(circle_at_68%_18%,rgba(255,242,178,0.56),transparent_15rem),linear-gradient(135deg,#69320d,#c68216_48%,#523209)]",
  },
  {
    author: "青山",
    likes: 88,
    ratio: "aspect-[9/16]",
    footer: "国风水墨头像，宣纸肌理，黑白留韵",
    kicker: "ink portrait",
    style:
      "bg-[radial-gradient(circle_at_54%_34%,#f7f3e8_0_9rem,transparent_9.5rem),linear-gradient(160deg,#f6efe0,#e1d3bd)]",
  },
  {
    author: "木叶",
    likes: 241,
    ratio: "aspect-[4/3]",
    footer: "忍者电影级特写，火光与烟雾穿透镜头",
    kicker: "cinematic character",
    style:
      "bg-[radial-gradient(circle_at_51%_39%,rgba(255,126,56,0.8),transparent_4rem),radial-gradient(circle_at_60%_13%,rgba(208,255,255,0.32),transparent_7rem),linear-gradient(135deg,#07131a,#27333c_45%,#030507)]",
  },
  {
    author: "小夏",
    likes: 126,
    ratio: "aspect-[9/16]",
    footer: "校园写真感少女，洞穴天光，自然胶片",
    kicker: "daily photo",
    style:
      "bg-[radial-gradient(circle_at_62%_30%,rgba(255,255,255,0.9),transparent_6rem),linear-gradient(135deg,#d8f3ff,#f5f8ff_45%,#b6a08c)]",
  },
  {
    author: "南枝",
    likes: 73,
    ratio: "aspect-square",
    footer: "古风人物，红妆珠钗，浅景深摄影",
    kicker: "portrait style",
    style:
      "bg-[radial-gradient(circle_at_38%_28%,rgba(255,206,213,0.58),transparent_6rem),linear-gradient(135deg,#2b0c12,#8a1f2d_45%,#151114)]",
  },
  {
    author: "工造社",
    likes: 56,
    ratio: "aspect-[4/3]",
    footer: "机械设定图，产品蓝图，硬表面设计",
    kicker: "blueprint",
    style:
      "bg-[linear-gradient(90deg,rgba(20,28,38,0.08)_1px,transparent_1px),linear-gradient(rgba(20,28,38,0.08)_1px,transparent_1px),linear-gradient(135deg,#f2f4f0,#d4d7ce)] bg-[length:28px_28px,28px_28px,auto]",
  },
];

export default function HomePage() {
  const [activeTab, setActiveTab] = useState("灵感");
  const [userPanelOpen, setUserPanelOpen] = useState(false);
  const [settings, setSettings] = useState<PublicSettings>({
    logoUrl: "",
    siteDescription: "面向前台用户的 AI 创作首页、会员入口和内容展示配置。",
    siteName: "即梦工作台",
    siteSlogan: "开启你的 Agent 模式，即刻造梦！",
  });

  useEffect(() => {
    const publicApiBase = process.env.NEXT_PUBLIC_POP_TAIL_PUBLIC_API_BASE?.trim() || "";
    const settingsUrl = `${publicApiBase.replace(/\/$/, "")}/public/frontend/settings`;
    fetch(settingsUrl, {
      credentials: "omit",
      headers: {
        Accept: "application/json",
      },
      method: "GET",
    })
      .then((response) => response.json())
      .then((payload) => {
        if (payload?.code === 0 && payload.data) {
          setSettings(payload.data);
        }
      })
      .catch(() => {
        // Public settings are optional; keep the built-in design defaults.
      });
  }, []);

  return (
    <main className="min-h-screen bg-[#070809] text-[#f4f7fb]">
      <aside className="fixed inset-y-0 left-0 z-40 hidden w-[76px] flex-col items-center border-r border-white/[0.03] bg-[#080a0c] pb-5 pt-7 md:flex">
        <div className="mb-20 size-8 rotate-45 rounded-[40%_8%_40%_8%] bg-[linear-gradient(135deg,#f7ff6a,#07d9ff_48%,#7a6cff)] shadow-[0_0_28px_rgba(0,224,255,0.45)]" />
        <nav className="flex flex-1 flex-col items-center gap-7">
          {sidebarItems.map((item) => (
            <button
              className={cn(
                "group flex w-full flex-col items-center gap-1.5 text-[12px] transition",
                item.active ? "text-white" : "text-[#8b929d] hover:text-white",
              )}
              key={item.label}
            >
              <item.icon className="size-6 stroke-[2.4]" />
              <span>{item.label}</span>
            </button>
          ))}
          <button className="mt-2 flex flex-col items-center gap-1.5 text-[12px] text-[#8b929d] transition hover:text-white">
            <span className="grid size-7 place-items-center rounded-lg bg-white text-black">O</span>
            <span>Octo</span>
            <span className="-mt-1 rounded bg-cyan-400/16 px-1 text-[10px] text-cyan-300">Beta</span>
          </button>
        </nav>
        <div className="flex flex-col items-center gap-5 text-[#8b929d]">
          <button className="rounded-xl border border-cyan-400/20 px-2 py-2 text-[11px] text-cyan-300">
            ✦ 66
            <br />
            1元会员
          </button>
          <button
            className="relative size-8 overflow-hidden rounded-full bg-[radial-gradient(circle_at_35%_30%,#fff,#8390a4_45%,#303643)]"
            onClick={() => setUserPanelOpen((open) => !open)}
          >
            <span className="sr-only">user</span>
          </button>
          <button className="relative">
            <Upload className="size-5" />
            <span className="absolute -right-2 -top-2 rounded-full bg-cyan-400 px-1 text-[10px] text-black">6</span>
          </button>
          <button className="text-sm font-semibold">API</button>
          <button>
            <Menu className="size-5" />
          </button>
        </div>
      </aside>

      <section className="min-h-screen pl-0 md:pl-[76px]">
        <header className="relative flex h-[142px] items-center justify-center px-5">
          <h1 className="text-center text-2xl font-black tracking-[0.02em] text-white sm:text-[32px]">
            {settings.siteSlogan.includes("Agent") ? "开启你的" : settings.siteName}{" "}
            <span className="bg-[linear-gradient(90deg,#20e7ff,#00b7c7)] bg-clip-text text-transparent">
              Agent 模式
            </span>
            <ChevronDown className="mx-1 inline size-6 text-cyan-400" />
            即刻造梦！
          </h1>
          <button
            className="absolute right-5 top-7 hidden items-center gap-2 rounded-full border border-[#2b3038] bg-[#12151b] px-3 py-1.5 text-sm text-white hover:bg-[#191d25] md:flex"
            onClick={() => setUserPanelOpen((open) => !open)}
          >
            <span className="size-6 rounded-full bg-[radial-gradient(circle_at_35%_30%,#fff,#8390a4_45%,#303643)]" />
            苹果🍎
          </button>
          {userPanelOpen ? (
            <div className="absolute right-5 top-18 z-50 w-72 rounded-2xl border border-white/10 bg-[#171922] p-4 text-left shadow-2xl">
              <div className="flex items-center gap-3">
                <span className="size-12 rounded-full bg-[radial-gradient(circle_at_35%_30%,#fff,#ff9ab0_45%,#c92654)]" />
                <div>
                  <div className="font-bold text-white">苹果🍎</div>
                  <div className="text-xs text-[#8b929d]">普通会员 · 今日灵感 66</div>
                </div>
              </div>
              <div className="mt-4 grid grid-cols-3 gap-2 text-center">
                <div className="rounded-xl bg-white/[0.04] p-2">
                  <div className="font-bold">18</div>
                  <div className="text-[11px] text-[#8b929d]">作品</div>
                </div>
                <div className="rounded-xl bg-white/[0.04] p-2">
                  <div className="font-bold">6</div>
                  <div className="text-[11px] text-[#8b929d]">消息</div>
                </div>
                <div className="rounded-xl bg-white/[0.04] p-2">
                  <div className="font-bold">API</div>
                  <div className="text-[11px] text-[#8b929d]">开放</div>
                </div>
              </div>
              <p className="mt-4 text-xs leading-5 text-[#8b929d]">{settings.siteDescription}</p>
            </div>
          ) : null}
        </header>

        <div className="mx-auto max-w-[1008px] px-4 pb-10">
          <section className="rounded-[20px] bg-[#171922] px-4 pb-4 pt-4 shadow-[0_20px_56px_rgba(0,0,0,0.25)]">
            <div className="flex min-h-[77px] items-start gap-4">
              <button className="grid size-[52px] rotate-[-8deg] place-items-center rounded-[4px] bg-[#292d39] text-[#78808d] shadow-xl transition hover:rotate-0 hover:bg-[#313645]">
                <Plus className="size-5" />
              </button>
              <div className="pt-1 text-sm text-[#747b87]">agent_prompt_noref</div>
            </div>
            <div className="flex items-center gap-1.5 overflow-x-auto pt-1">
              <button className="flex h-8 shrink-0 items-center gap-1.5 rounded-lg border border-[#303440] bg-[#11141b] px-3 text-xs text-cyan-300">
                <span className="text-sm">///</span>
                Agent 模式
                <ChevronDown className="size-3" />
              </button>
              <button className="flex h-8 shrink-0 items-center gap-1.5 rounded-lg border border-[#303440] px-3 text-xs text-[#d6dae2]">
                <SlidersHorizontal className="size-4" />
                自定义
              </button>
              <button className="flex h-8 min-w-[96px] shrink-0 items-center gap-1.5 rounded-lg border border-[#303440] px-3 text-xs text-[#757c88]">
                <Search className="size-4" />
                灵感搜索
              </button>
              <button className="flex h-8 shrink-0 items-center gap-1.5 rounded-lg border border-[#303440] px-3 text-xs text-[#757c88]">
                <WandSparkles className="size-4" />
                创意设计
              </button>
              <button className="ml-auto grid size-8 shrink-0 place-items-center rounded-full bg-[#343946] text-sm text-[#858d9b]">
                ↑
              </button>
            </div>
          </section>

          <section className="mt-8 grid gap-2 sm:grid-cols-2 xl:grid-cols-5">
            {tools.map((tool) => (
              <button className="flex h-[58px] items-center gap-2.5 rounded-[14px] bg-[#14161d] px-3.5 text-left transition hover:-translate-y-1 hover:bg-[#191c25]" key={tool.label}>
                <span className={cn("grid size-[33px] shrink-0 place-items-center rounded-[10px] text-xs font-black", tool.badgeClass)}>
                  {tool.badge || <Sparkles className="size-4 text-white" />}
                </span>
                <span>
                  <span className="flex items-center gap-1.5 text-sm font-bold text-white">
                    {tool.label}
                    {tool.tag ? <span className="rounded bg-cyan-400/16 px-1 py-0.5 text-[10px] text-cyan-300">{tool.tag}</span> : null}
                  </span>
                  <span className="mt-0.5 block text-xs text-[#7e8490]">{tool.meta}</span>
                </span>
              </button>
            ))}
          </section>
        </div>

        <section className="mx-auto mt-32 max-w-[1816px] px-4 pb-10 md:px-10">
          <div className="flex flex-col gap-4 md:flex-row md:items-center">
            <div className="flex rounded-lg bg-[#171b24] p-0.5">
              {tabs.map((tab) => (
                <button
                  className={cn(
                    "h-9 rounded-md px-5 text-sm font-bold transition",
                    activeTab === tab ? "bg-[#242b38] text-white" : "text-[#8b929d] hover:text-white",
                  )}
                  key={tab}
                  onClick={() => setActiveTab(tab)}
                >
                  {tab}
                </button>
              ))}
            </div>
            <label className="flex h-9 w-full max-w-[320px] items-center gap-2 rounded-lg border border-[#2c313d] px-3.5 text-[#8c939f] md:ml-5">
              <Search className="size-4" />
              <input className="w-full bg-transparent text-xs outline-none placeholder:text-[#8c939f]" placeholder="露脚的美女全身照" />
            </label>
          </div>

          <div className="mt-6 columns-1 gap-0 sm:columns-2 lg:columns-4">
            {gallery.map((item, index) => (
              <article className={cn("group relative mb-0 break-inside-avoid overflow-hidden bg-[#151820]", item.ratio)} key={`${item.footer}-${index}`}>
                <div className={cn("absolute inset-0 transition duration-500 group-hover:scale-105", item.style)} />
                <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/72 via-black/24 to-transparent p-6">
                  <p className="text-sm font-semibold text-white/80">{item.kicker}</p>
                  <h2 className="mt-2 max-w-2xl text-xl font-black leading-tight text-white">{item.footer}</h2>
                  {index === 0 ? (
                    <div className="mt-5 flex gap-1">
                      <span className="size-2 rounded-full bg-white/70" />
                      <span className="size-2 rounded-full bg-white/35" />
                      <span className="size-2 rounded-full bg-white/35" />
                    </div>
                  ) : null}
                </div>
                {index === 0 ? (
                  <div className="absolute left-1/2 top-8 -translate-x-1/2 text-center">
                    <div className="text-sm font-bold text-white/75">✦ 即梦AI ｜ 学院奖</div>
                    <div className="mt-5 text-xl font-black tracking-wide text-white">大学生广告艺术节学院奖 即梦AI青年创意大赛</div>
                  </div>
                ) : null}
                <div className="absolute inset-x-0 bottom-0 z-20 flex translate-y-3 items-center justify-between bg-gradient-to-t from-black/88 via-black/52 to-transparent px-5 pb-4 pt-12 opacity-0 transition duration-300 group-hover:translate-y-0 group-hover:opacity-100">
                  <div className="flex min-w-0 items-center gap-2.5">
                    <span className="grid size-8 shrink-0 place-items-center overflow-hidden rounded-full bg-[radial-gradient(circle_at_32%_28%,#fff,#ff9ab0_42%,#c92654)] text-sm">
                      {item.author.slice(0, 1)}
                    </span>
                    <span className="truncate text-base font-semibold text-white">{item.author}</span>
                  </div>
                  <div className="flex shrink-0 items-center gap-5 text-white">
                    <button className="text-white drop-shadow-[0_2px_6px_rgba(0,0,0,0.55)] transition hover:scale-110" aria-label="复用">
                      <CopyPlus className="size-8 stroke-[2.2]" />
                    </button>
                    <button className="text-white drop-shadow-[0_2px_6px_rgba(0,0,0,0.55)] transition hover:scale-110" aria-label="查看图片">
                      <ImageIcon className="size-8 stroke-[2.2]" />
                    </button>
                    <button className="flex items-center gap-1.5 text-white drop-shadow-[0_2px_6px_rgba(0,0,0,0.55)] transition hover:scale-105" aria-label="喜欢">
                      <Heart className="size-8 stroke-[2.2]" />
                      <span className="text-2xl font-semibold">{item.likes}</span>
                    </button>
                  </div>
                </div>
              </article>
            ))}
          </div>
        </section>
      </section>

    </main>
  );
}
