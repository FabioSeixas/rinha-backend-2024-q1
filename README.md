### Rinha Backend 2024 Q1

https://github.com/zanfranceschi/rinha-de-backend-2024-q1/tree/main

**Current Progress**

Still having some validation errors:

![image](https://github.com/FabioSeixas/rinha-backend-2024-q1/assets/43079786/bd442c18-55b1-4996-8b1d-89d0210fcadb)

Ideas:
1. I'am starting the transaction after checking if the user exists. I should include this first check in the transaction.
2. Manage transactions from Postgres (use functions `debitar` and `creditar` from `init.sql`)
3. Try different combinations of CPU and Memory for services (`docker-compose.yml`)
4. Possible error in my logic?

