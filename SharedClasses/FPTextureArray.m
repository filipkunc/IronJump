//
//  FPTextureArray.m
//  IronJump
//
//  Created by Filip Kunc on 6/1/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import "FPTextureArray.h"

@implementation FPTextureArray

- (id)init
{
	self = [super init];
	if (self)
	{
		textures = [[NSMutableArray alloc] init];		 
	}
	return self;
}

- (void)dealloc
{
	[textures release];
	[super dealloc];
}

- (NSUInteger)count
{
	return [textures count];
}

- (void)addTexture:(NSString *)fileName
{
	FPTexture *texture = [[FPTexture alloc] initWithFile:fileName convertToAlpha:NO];
	[textures addObject:texture];
	[texture release];
}

- (FPTexture *)textureAtIndex:(NSUInteger)index
{
	return (FPTexture *)[textures objectAtIndex:index];
}

@end
