`git wire sync` / `git wire check` において、`.gitwire` エントリ間で
`url` + `rev` + `mtd` がすべて同一のエントリが複数存在する場合、
一時クローン（TempDir）を1つだけ作成して共有して処理したい。

- seuquence::parallel 処理の

    - parsed の内容から url, rev, mtd キーでユニークな配列を生成(fetches)

    - spawn 処理を２段階にする
        - １段階目
            - fetches を並行して実行
            - ログの previx は上記のキー利用する
        - ２段階目
            - operation (sync or check) を並行して実行

    - 1 段階目と２段階目は channel(s) で接続する
        - ２段階目の処理に対応する tx, rx を生成しておく
        - １段階目には配列でこれらの tx を渡す
        - ２段階目には rx を渡す
        - １段階目の最後の処理や失敗時に必ず tx(複数) に情報(Result)を渡す
            - ２段階目でエラー処理を行う




