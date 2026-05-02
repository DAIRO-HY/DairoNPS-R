
delete from channel where 1=1;
delete from client where 1=1;
delete from forward where 1=1;
delete from traffic_stats where 1=1;

insert into client(id,name,key)values(1,'客户端1','njeHds*fs4tfsd');
insert into client(id,name,key)values(2,'客户端2','njeHds*fs4tfs');

insert into channel(id,client_id,name,server_port,target_port,mode,is_stats_traffic)values (7,1,'隧道9090',8090,'8080',1,0);
insert into channel(id,client_id,name,server_port,target_port,mode)values (8,1,'隧道9091',8091,'8080',1);
insert into channel(id,client_id,name,server_port,target_port,mode)values (9,2,'隧道9092',8092,'8080',1);

insert into forward(id,name, server_port, target_port,created_at,is_stats_traffic)values (4,'端口转发001',9090,'127.0.0.1:8080',0,0);
insert into forward(id,name, server_port, target_port,created_at)values (5,'端口转发002',9091,'127.0.0.1:8080',0);
insert into forward(id,name, server_port, target_port,created_at)values (6,'端口转发003',9092,'127.0.0.1:8080',0);

update system_config set in_len = 0,out_len = 0 where 1 = 1;

