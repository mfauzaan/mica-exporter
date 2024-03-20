FROM public.ecr.aws/lambda/provided:al2-arm64

COPY ./target/lambda/mica-exporter/bootstrap ${LAMBDA_RUNTIME_DIR}

# Set the CMD to your handler (could also be done as a parameter override outside of the Dockerfile)
CMD [ "hello.handler" ]
