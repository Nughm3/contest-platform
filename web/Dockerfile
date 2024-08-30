FROM node:slim AS build

WORKDIR /build

# Set up pnpm
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable 

# Install dependencies
RUN rm -rf node_modules
COPY package.json package.json
COPY pnpm-lock.yaml pnpm-lock.yaml
RUN pnpm install

# Copy sources and build
COPY . .
RUN pnpm build

FROM node:slim AS final

WORKDIR /web

# Copy build output
COPY --from=build /build/build build
COPY --from=build /build/node_modules node_modules
COPY package.json .

# Configure
ENV NODE_ENV=production
EXPOSE 3000

# Run
CMD ["node", "build"]
