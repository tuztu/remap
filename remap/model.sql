create table if not exists `user` (
    `id` bigint unsigned not null,
    `name` varchar(64) not null,
    primary key (`id`)
) engine = InnoDB;