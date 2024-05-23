# Solana query service

作为一个博客访客，
我想订阅电子报，
以便在博客上发布新内容时收到电子邮件更新。
我们希望我们的博客访客在嵌入在网页上的表单中输入他们的电子邮件地址。
该表单将触发一个POST /subscriptions调用我们的后端API来处理信息、存储它并返回响应。
我们将需要深入了解以下内容：

如何在actix-web中读取在HTML表单中收集的数据（即如何解析POST请求的请求体）；
用于在Rust中使用PostgreSQL数据库的可用库（diesel vs sqlx vs tokio-postgres）；
