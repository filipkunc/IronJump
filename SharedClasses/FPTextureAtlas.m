//
//  FPTextureAtlas.m
//  IronJump
//
//  Created by Filip Kunc on 4/17/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import "FPTextureAtlas.h"

@implementation FPTextureAtlas

@synthesize texture, tileSize;

- (id)initWithFile:(NSString *)fileName convertToAlpha:(BOOL)convertToAlpha tileSize:(int)aTileSize
{
	self = [super init];
	if (self)
	{
		texture = [[FPTexture alloc] initWithFile:fileName convertToAlpha:convertToAlpha];
		tileSize = aTileSize;
	}
	return self;
}

- (int)horizontalTileCount
{
	return [texture width] / tileSize;
}

- (int)verticalTileCount
{
	return [texture height] / tileSize;
}

@end
