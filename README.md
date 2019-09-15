# Auto Bencher 自動 Benchmark 君

## 功能

- `auto-bencher init-env`
    - 初始化與檢察環境
- (not yet) `auto-bencher clean`
    - 清空環境，刪除所有此程式建立的檔案
- (not yet) `auto-bencher execute-cmd [CMD]`
    - 執行給定指令 `CMD`
- `auto-bencher load [db name] [parameter file]`
    - 為給定數量的機器跑載入資料
- (not yet) `auto-bencher check-ready`
    - 檢查環境是否該有的東西都有，包括 benchmark 的資料都準備好了
- `auto-bencher bench [db name] [parameter file]`
    - 用給定的參數跑 benchmarks

### Refactoring

- Improve the error handling (stack trace ?)

### Bug To Fix

- Delete java runtime zip on the machines

### 期望功能

- ~~必須要檢查 server 跟 client 是否都正常啟動~~
- 能夠偵測 exception 以外的錯誤 (程式意外終止)
- 能夠自動抓取 properties 產生 parameter file
- 能夠合併 csv report，並另外產生一個 total summary 的 csv
- log 先產生在外面，等到跑完之後再一併放回 results
- CPU 監測及即時繪圖
- throughput 即時繪圖
- 提供一個功能是在每個 stage 安插指令
- ~~應該在 loading 時提供要 load 的機器數，config 只能設定總共有幾台機器~~
- ~~只需要在一個 mapping table 內加入 tunable parameter，就可以自動產生 properties files~~
- 就算沒跑完也要把 client report 或是 benchmark report 拉回來
- 禁止在 parameter file 中設定會被 auto bencher 寫入的 property (例如 `STAND_ALONE_SEQUENCER`)