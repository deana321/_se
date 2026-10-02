# HW4：分支、合併、Fork、Pull Request 實作紀錄

使用opencode協助

## 我的三個連結

* 母專案：https://github.com/nqusc/115se/commits/main
* 分支：https://github.com/nqusc/115se/commits/developGitBranch
* 子專案（Fork）：https://github.com/deana321/developGitBranch/commits/main

> 子專案頁面有寫 `forked from nqusc/115se`，證明是 Fork 來的。

---

## 1. Initial commit（專案初始存檔）

在 GitHub 上按 `Create new repository` 建立新專案，owner 選 `nqusc`，名稱叫 `115se`。

這就是第一筆 `Initial commit`。

---

## 2. add gitBranch.md（本地分支開發與合併）

做法：

```bash
git checkout -b developGitBranch
```

建立一個 `gitBranch.md` 檔案，然後：

```bash
git add gitBranch.md
git commit -m "add gitBranch.md"
git checkout main
git merge developGitBranch --no-ff -m "merge developGitBranch"
git push origin main
git push origin developGitBranch
```

這樣 `developGitBranch` 分支就多了一個 `add gitBranch.md`，
`main` 用合併的方式也拿到了這個檔案。

驗證：

```bash
git branch -a
git log --oneline --graph --all -10
```

---

## 3. add deana321Fork.md（Fork 練習檔案）

1. 先到母專案 `https://github.com/nqusc/115se` 按 `Fork` → `Create fork` 到 `deana321`。
   * 母專案要先在 `Settings` 打開 `Allow forking`，不然會顯示 `forking is disabled`。
   * 我 Fork 後的名字是 `deana321/developGitBranch`。
2. 把 Fork 下載回電腦：

```bash
git clone https://github.com/deana321/developGitBranch.git C:\Temp\fork-demo
cd C:\Temp\fork-demo
```

3. 新增 Fork 專用檔案並上傳：

```bash
echo "fork by me" > deana321Fork.md
git add deana321Fork.md
git commit -m "add deana321Fork.md"
git push origin main
```

這時子專案有 `deana321Fork.md`，母專案還沒有，
跟老師範例 `ccckmitFork.md` 的情況一樣。

---

## 4. Merge pull request（完成 PR 合併）

1. 打開子專案 `https://github.com/deana321/developGitBranch`
2. 按 `Contribute` → `Open pull request`
3. 左邊選 `nqusc/115se : main`，右邊選 `deana321/developGitBranch : main`
4. 按 `Create pull request`
5. 在母專案的 PR 頁按綠色 `Merge pull request` → `Confirm merge`

合併後母專案 `main` 會多一筆 `Merge pull request from deana321`，
就跟範例的 `Merge pull request #1 from ccckmit/main` 一樣。

---

## 5. 這是哪一種 Git 流程？

對照阮一峰《Git 工作流程》：

* 有 `main` + `developGitBranch` 兩個長期分支 → 像 **Git flow**
* 用 PR 合併、看完 code 再按 Merge → 像 **GitHub flow**
* 有分 `upstream（nqusc/115se）` 和 `fork（deana321/developGitBranch）`，只能從 fork 發 PR 回上游 → 是 **GitLab flow** 說的 `upstream first`

結論：三種混在一起用，最接近 **GitLab flow**。

參考：

* https://www.ruanyifeng.com/blog/2015/12/git-workflow.html
* https://bytebytego.com/guides/how-does-git-work/
