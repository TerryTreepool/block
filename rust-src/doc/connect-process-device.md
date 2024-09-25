 
# device box
## leave of factory

```sequence
    participant thing as a
    participant device as b
    participant service as c
```

## startup

```sequence
    participant thing as a
    participant device as b
    participant service as c

    b -> b: listen endpoint
    b -> c: try connect
    note over b, c: exchange key
    b -> c: exchange key
    c -> b: ack
    b -> c: ackack

    note over b, c: syn tunnel
    b -> c: syn device tunnel info
    c -> b: ack
    b -> c: ackack

    b -> b: collect local device
    b -> a: try connect
```
> **约定**
> * box启动后, 与service同步密钥和设备信息;
> * 统计与box链接过的所有thing, 并尝试链接它们;


## box to box
```sequence
    participant device1 as b1
    participant service as c
    participant device2 as b2

    b1 -> c: create tunnel
    b2 -> c: create tunnel

    b1 -> c: exchange b2 tunnel info
    c -> b2: sync tunnel
    b1 -> b1: check protocol between device1 and device2

    note over b1: device1 and device2 in the same LAN;
    b1 -> b2: exchange tunnel info by local endpoint

    note over b1: device1 and device2 not in the same LAN;
    b1 -> b2: exchange tunnel info by remote endpoint

```
> **约定**
> * b1, b2 所属Owner必须是相同, 暂不放开;
> * 如果相同局域网, 优先使用本地地址端口链接;
> * 非同一局域网, 使用外网地址端口链接;


# user app
## register
```sequence
    participant user as a
    participant app as b
    participant device as c

    b -> b: startup
    b -> c: create tunnel

    a -> b: scan QR of b when register
    a -> a: input user info(eg: name, password etc.)
    b -> b: check tunnel between app and box
    b -> b: create user object
    b -> a: sync user object
    a -> a: cache user object

```

> **约定**
> * 用户信息暂时不同步外网, 只保留在本地;
> * 用户通过注册时的用户名/密码登录app;
> * box如果重装系统后, 手机依然可以通过app同步用户信息到box端, 因为userObject是可信任的;
> * 如果手机资料清空, 可以通过用户名+密码登录app, 并从box中拿回UserObject信息;
> * 用户注册后, 在存在UserObject的情况下, 不需要登录;
> * 用户可以选择性登出, 登出后, 可以增加解锁密码或者手势解锁灯;
> 

## login
```sequence
    participant user as a
    participant app as b
    participant device as c

    b -> b: startup
    b -> c: create tunnel

    a -> a: check userObject
    note over a, b: found UserObject
    a -> b: login with password;
    note over a, b: no UserObject found
    a -> b: login with name + password
    b -> b: check user, password invalid
    b -> b: check expire time
    b -> a: sync userObject
    b -> a: exchange boxObject
```
> **约定**
> * 一般情况下, 不需要用户主动登录;
> * app端查找userObject资料, 如果有, 使用password登录;
> * 如果没有, 使用name+password登录;
> 