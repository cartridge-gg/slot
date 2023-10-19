![Slot Cover Image](.github/cover.png)

# Slot

A toolchain for rapidly spinning up Katana and Torii instances. Play test your game in seconds.

## Installation

Install `slotup` to manage slot installations and follow the outputted directions.
```
curl -L https://slot.cartridge.sh | bash
```

## Usage

Authenticate with Cartridge
```
slot auth login
```

Create service deployments
```
slot deployments create <Project Name> katana
slot deployments create <Project Name> torii --world 0x3fa481f41522b90b3684ecfab7650c259a76387fab9c380b7a959e3d4ac69f
```

Read service logs
```
slot deployments logs <Project Name> <katana | torii>
```

List all deployments
```
slot deployments list
```

View deployments configuration
```
slot deployments describe <Project Name> <katana | torii>
```