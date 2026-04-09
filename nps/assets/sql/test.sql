
delete from channel where 1=1;
delete from client where 1=1;
delete from channel_data where 1=1;

insert into client(id,name,key)values(1,'客户端1','njeHds*fs4tfsd');
insert into client(id,name,key)values(2,'客户端2','njeHds*fs4tfs');

insert into channel(id,client_id,name,server_port,target_port,mode)values (7,1,'隧道9090',9090,'8080',1);
insert into channel(id,client_id,name,server_port,target_port,mode)values (8,1,'隧道9091',9091,'8080',1);
insert into channel(id,client_id,name,server_port,target_port,mode)values (9,2,'隧道9092',9092,'8080',1);

update system_config set in_len = 0,out_len = 0 where 1 = 1;
