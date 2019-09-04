FROM enigmampc/enigma-core-base

LABEL maintainer=enigmampc

ARG SGX_MODE

ENV SGX_MODE=${SGX_MODE:-SW}

COPY . .

WORKDIR /root/enigma-principal

RUN make DEBUG=1

RUN mkdir -p $HOME/.enigma 

WORKDIR /root

CMD ["./execute_km.sh"]