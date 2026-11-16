// 功能：在文档根节点出现时安装轻量滚动条样式，随页面导航重建且不监听滚动；时间：2026-11-16；作者：lq。
(() => {
  const install = () => {
    if (!document.documentElement) return false;
    const style = document.createElement('style');
    style.textContent = `
      ::-webkit-scrollbar{width:10px;height:10px}
      ::-webkit-scrollbar-track,::-webkit-scrollbar-corner{background:transparent}
      ::-webkit-scrollbar-thumb{background:rgba(105,115,130,.42);border:2px solid transparent;border-radius:999px;background-clip:padding-box}
      ::-webkit-scrollbar-thumb:hover{background-color:rgba(90,100,115,.7)}
      ::-webkit-scrollbar-button{display:none}
      @media(prefers-color-scheme:dark){::-webkit-scrollbar-thumb{background-color:rgba(170,180,195,.42)}::-webkit-scrollbar-thumb:hover{background-color:rgba(190,200,215,.7)}}
      @supports not selector(::-webkit-scrollbar){*{scrollbar-width:thin;scrollbar-color:rgba(120,130,145,.6) transparent}}
      @media(forced-colors:active){*{scrollbar-color:auto}::-webkit-scrollbar-thumb{background:CanvasText;border-color:Canvas}}
    `;
    document.documentElement.appendChild(style);
    return true;
  };
  if (install()) return;
  // 功能：仅等待初始根节点，立即断开观察，不建立持续 DOM 或滚动监听；时间：2026-11-16；作者：lq。
  const observer = new MutationObserver(() => { if (install()) observer.disconnect(); });
  observer.observe(document, { childList: true });
})()
