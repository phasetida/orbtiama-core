## orbtiama-core
[WIP]一个简单的Rust库，将SiMai格式的谱面转为紧凑的结构体  
本仓库目前处于开发阶段，功能尚不稳定
## 进度
- [ ] 解析谱面
  - [x] Tap note
  - [x] Touch note
  - [x] Hold note
    - [x] Tap hold note
    - [x] Touch hold note
      - [ ] 假Hold语法
  - [x] Slide note
    - [x] 路径
      - [x] - 路径
      - [x] < ^ > 路径
      - [x] p q 路径
      - [x] pp qq 路径
      - [x] s z 路径
      - [x] v 路径
      - [x] V 路径
      - [x] w 路径
    - [ ] 单头多尾（未测试）
    - [x] 连环
  - [x] Note Decoration
  - [x] Each 假Each
  - [ ] 测试用例
- [ ] 渲染谱面
  - [ ] Ticking
    - [x] Tap note
    - [x] Touch note
    - [x] Hold note
      - [x] Tap hold note
      - [x] Touch hold note
    - [x] Slide note
  - [ ] Drawing
    - [x] Tap note
    - [x] Touch note
    - [x] Hold note
    - [ ] Slide note
      - [x] - 路径
      - [x] < ^ > 路径
      - [x] p q 路径
      - [x] pp qq 路径
      - [x] s z 路径
      - [x] v 路径
      - [x] V 路径
      - [ ] w 路径
    - [ ] Note decoration
      - [ ] Break
      - [ ] Ex
      - [ ] Fireworks
      - [ ] Mine
      - [ ] SuddenIn
      - [ ] FadeOut
      - [ ] Each
      - [ ] ForceStar
      - [ ] Spinning
      - [ ] ForceNormal
  - [x] 基础序列化

## 备注
在未来（如果有机会的话），谱面文件的解析相关部分将会分离至单独的仓库