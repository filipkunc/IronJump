//
//  FPTextureArray.h
//  IronJump
//
//  Created by Filip Kunc on 6/1/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
//

#import "FPTexture.h"

@interface FPTextureArray : NSObject
{
	NSMutableArray *textures;
}

@property (readonly) NSUInteger count;

- (void)addTexture:(NSString *)fileName;
- (FPTexture *)textureAtIndex:(NSUInteger)index;

@end
