// 功能：启动桌面宿主，发行版隐藏 Windows 控制台；时间：2026-11-08；作者：lq。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    sab_lib::run();
}
