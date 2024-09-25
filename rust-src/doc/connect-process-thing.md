
# connect box
```sequence
    participant user
    participant thing as a
    participant device as b
    participant service as c

    user -> a: set device-info in thing?
    a -> a: listen, get local address

    note over b, a: connect failure
    a -> a: connect failed
    a -> c: try connect
    a -> c: sync thing info( local-endpoint, remote-endpoint )
    a -> c: i want to device info
    c -> a: sync device info
    c -> b: sync thing info

    a -> b: try connect with local-endpoint
    b -> a: reverse connect with local-endpoint
```
> thing与box链接, 只支持与local-endpoint链接; 不开放与remote-endpoint链接;