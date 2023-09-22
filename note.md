# Some attempt to check proper sequence of checkout methods

## shallow checkout with sparse-checkout

It worked.

 ```sh
 git clone --depth 1 --filter=blob:none --no-checkout https://github.com/msr1k/git-wire.git
 cd git-wire
 # git sparse-checkout set src/common/ # It doesn't work
 git sparse-checkout init
 echo "/src/common/" > .git/info/sparse-checkout
 git fetch --depth 1 origin 3ea97596a309dc0e1fd317f4bbaffccb0d455a49
 git checkout FETCH_HEAD
```

