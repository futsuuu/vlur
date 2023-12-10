if v:errmsg == ''
    quitall!
else
    echo ''
    cquit! 1
endif
